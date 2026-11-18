use crate::{Step, install, summary};
use std::cell::Cell;
use std::io;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use windows_sys::Win32::Foundation::{HWND, S_FALSE, S_OK};
use windows_sys::Win32::UI::Controls::*;
use windows_sys::Win32::UI::WindowsAndMessaging::{IDOK, SendMessageW};
use windows_sys::w;

enum Update {
    Step(String),
    Done(bool, String),
}

struct Work {
    root: PathBuf,
    steps: Vec<Step>,
    updates: Sender<Update>,
}

struct State {
    work: Cell<Option<Work>>,
    updates: Receiver<Update>,
    result: Cell<Option<bool>>,
}

pub fn run(root: PathBuf, steps: Vec<Step>) -> io::Result<bool> {
    let (send, updates) = mpsc::channel();
    let state = State {
        work: Cell::new(Some(Work { root, steps, updates: send })),
        updates,
        result: Cell::new(None),
    };
    let config = TASKDIALOGCONFIG {
        cbSize: size_of::<TASKDIALOGCONFIG>() as u32,
        dwFlags: TDF_CALLBACK_TIMER,
        dwCommonButtons: TDCBF_OK_BUTTON,
        pszWindowTitle: w!("课程环境安装"),
        pszMainInstruction: w!("正在安装课程环境"),
        pszContent: w!("正在准备安装…"),
        cxWidth: 200,
        pfCallback: Some(on_event),
        lpCallbackData: &raw const state as isize,
        ..Default::default()
    };
    // SAFETY: 配置、文本和回调状态在模态对话框关闭前始终有效。
    let status = unsafe { TaskDialogIndirect(&config, null_mut(), null_mut(), null_mut()) };
    if status < 0 {
        return Err(io::Error::from_raw_os_error(status));
    }
    state.result.get().ok_or_else(|| io::Error::other("安装未返回结果"))
}

unsafe extern "system" fn on_event(hwnd: HWND, event: u32, _: usize, _: isize, data: isize) -> i32 {
    // SAFETY: data 来自 run，回调在对话框所属线程执行。
    let state = unsafe { &*(data as *const State) };
    match event as i32 {
        TDN_CREATED => {
            enable_ok(hwnd, false);
            if let Some(work) = state.work.take()
                && let Err(err) = thread::Builder::new().spawn(move || work.install())
            {
                state.finish(hwnd, false, &format!("无法启动安装：{err}"));
            }
        }
        TDN_TIMER if state.result.get().is_none() => state.refresh(hwnd),
        TDN_BUTTON_CLICKED if state.result.get().is_none() => return S_FALSE,
        _ => {}
    }
    S_OK
}

impl Work {
    fn install(self) {
        let results = install(&self.root, self.steps, |text| {
            let _ = self.updates.send(Update::Step(text));
        });
        let (ok, report) = summary(results, true);
        let _ = self.updates.send(Update::Done(ok, report));
    }
}

impl State {
    fn refresh(&self, hwnd: HWND) {
        loop {
            match self.updates.try_recv() {
                Ok(Update::Step(text)) => set_text(hwnd, TDE_CONTENT, &text),
                Ok(Update::Done(ok, report)) => return self.finish(hwnd, ok, &report),
                Err(TryRecvError::Disconnected) => {
                    return self.finish(hwnd, false, "安装意外中断，未返回结果。");
                }
                Err(TryRecvError::Empty) => return,
            }
        }
    }

    fn finish(&self, hwnd: HWND, ok: bool, detail: &str) {
        self.result.set(Some(ok));
        let title = if ok { "安全安装完成" } else { "安装未全部成功" };
        set_text(hwnd, TDE_MAIN_INSTRUCTION, title);
        set_text(hwnd, TDE_CONTENT, detail);
        enable_ok(hwnd, true);
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

fn set_text(hwnd: HWND, element: TASKDIALOG_ELEMENTS, text: &str) {
    let text = wide(text);
    // SAFETY: 回调内同步发送，文本在调用期间有效。
    unsafe {
        SendMessageW(hwnd, TDM_SET_ELEMENT_TEXT as u32, element as usize, text.as_ptr() as isize)
    };
}

fn enable_ok(hwnd: HWND, enabled: bool) {
    // SAFETY: hwnd 是回调收到的对话框句柄。
    unsafe { SendMessageW(hwnd, TDM_ENABLE_BUTTON as u32, IDOK as usize, enabled as isize) };
}
