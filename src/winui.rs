use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoUninitialize,
};
use windows::Win32::UI::Shell::{
    CLSID_ProgressDialog, IProgressDialog, PROGDLG_NOCANCEL, PROGDLG_NOTIME,
};
use windows::core::{HSTRING, Result, w};

pub struct Progress {
    dialog: IProgressDialog,
    // 字段按声明顺序释放：先释放对话框，再退出 COM。
    _apartment: Apartment,
}

impl Progress {
    pub fn open() -> Result<Self> {
        // SAFETY: 同一线程初始化、使用并释放 COM 对象。系统用自己的线程刷新窗口。
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
            let apartment = Apartment;
            let dialog: IProgressDialog =
                CoCreateInstance(&CLSID_ProgressDialog, None, CLSCTX_INPROC_SERVER)?;
            dialog.SetTitle(w!("课程环境安装"))?;
            // 文件和注册表更新不支持中途取消，也无法准确估计剩余时间。
            dialog.StartProgressDialog(None, None, PROGDLG_NOCANCEL | PROGDLG_NOTIME, None)?;
            Ok(Self { dialog, _apartment: apartment })
        }
    }

    pub fn update(&self, completed: usize, total: usize, text: &str) {
        // SAFETY: HSTRING 在调用期间有效，界面更新失败不影响安装。
        unsafe {
            let _ = self.dialog.SetLine(1, &HSTRING::from(text), false, None);
            let _ = self.dialog.SetProgress64(completed as u64, total as u64);
        }
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        // SAFETY: 此时 COM 仍处于初始化状态。
        let _ = unsafe { self.dialog.StopProgressDialog() };
    }
}

struct Apartment;

impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: 仅在 CoInitializeEx 成功后构造，并在同一线程释放。
        unsafe { CoUninitialize() };
    }
}
