use windows::{
    core::Error,
    Win32::{Foundation, Graphics::Gdi},
};

/// A wrapper struct containing a slice.
pub struct CaptureData {
    bits: &'static [u8],
    hbitmap: Gdi::HBITMAP,
}

impl Drop for CaptureData {
    fn drop(&mut self) {
        unsafe { Gdi::DeleteObject(self.hbitmap) };
    }
}

impl CaptureData {
    /// Returns a raw slice containing copied bitmap bit values.
    ///
    /// The bits are stored as a one-dimensional array, in which one pixel consists of 3 adjacent \[B, G, R] values.
    pub fn get_bits(&self) -> &[u8] {
        self.bits
    }
}
/// A struct that contains and manages information required for screen capturing
///
/// This struct must be first initialized using the "new" constructor.
pub struct CaptureManager {
    top_left: (i32, i32),
    wh: (i32, i32),
    dc: Gdi::HDC,
    dc_mem: Gdi::HDC,
    window_handle: Foundation::HWND,
    bitmap_info: Gdi::BITMAPINFO,
}

impl CaptureManager {
    /// Creates a new capture manager.
    ///
    /// # Errors
    ///
    /// This method will usually return an error if window_handle is invalid.
    /// Other errors are also possible but very unlikely to happen.
    ///
    /// # Examples
    ///
    /// Let's assume that the screen resolution is 1000x1000px. In this example, the captured area will be a 500x500px square in the center of the screen.
    /// ```no_run
    /// use std::error::Error;
    /// use qshot::CaptureManager;
    ///
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     let window_handle = 0; // A handle to a window that should be captured (0 to capture the entire screen).
    ///     let top_left = (250, 250); // X and Y coordinates of the upper-left corner of the screen/window.
    ///     let wh = (500, 500); // Width and height of the area which should be captured.
    ///     
    ///     let manager = CaptureManager::new(window_handle, top_left, wh)?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(
        window_handle: isize,
        top_left: (i32, i32),
        wh: (i32, i32),
    ) -> Result<CaptureManager, Error> {
        let window_handle = Foundation::HWND(window_handle);
        let dc = unsafe { Gdi::GetDC(window_handle) };
        if dc.is_invalid() {
            return Err(Error::from_win32());
        }
        let dc_mem = unsafe { Gdi::CreateCompatibleDC(dc) };
        if dc_mem.is_invalid() {
            return Err(windows::core::Error::from_win32());
        }

        let bitmap_info = Gdi::BITMAPINFO {
            bmiHeader: Gdi::BITMAPINFOHEADER {
                biSize: std::mem::size_of::<Gdi::BITMAPINFOHEADER>() as u32,
                biWidth: wh.0,
                biHeight: -wh.1,
                biBitCount: 24,
                biCompression: 0,
                biPlanes: 1,
                biSizeImage: 0,
                biClrImportant: 0,
                biClrUsed: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
            },
            bmiColors: [Gdi::RGBQUAD {
                rgbRed: 0,
                rgbGreen: 0,
                rgbBlue: 0,
                rgbReserved: 0,
            }],
        };

        Ok(CaptureManager {
            top_left,
            wh,
            bitmap_info,
            dc,
            dc_mem,
            window_handle,
        })
    }
    /// Captures the screen using the information provided in the constructor. Returns [`CaptureData`] if succeed.
    ///
    /// # Errors
    ///
    /// This method will return an error if the targeted window is closed.
    /// In some cases, incorrect coordinates will also cause the method to fail.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::error::Error;
    /// use qshot::CaptureManager;
    ///
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     let manager = CaptureManager::new(0, (250, 250), (500, 500))?;
    ///
    ///     let res = manager.capture()?;
    ///     assert_eq!(res.get_bits().len(), 500 * 500 * 3);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn capture(&self) -> Result<CaptureData, Error> {
        unsafe {
            let mut bits = std::mem::MaybeUninit::<*mut u8>::uninit();
            let hbitmap = Gdi::CreateDIBSection(
                self.dc,
                &self.bitmap_info,
                Gdi::DIB_RGB_COLORS,
                bits.as_mut_ptr() as *mut *mut std::ffi::c_void,
                None,
                0,
            )?;
            let bits = bits.assume_init();

            Gdi::SelectObject(self.dc_mem, hbitmap);

            Gdi::BitBlt(
                self.dc_mem,
                0,
                0,
                self.wh.0,
                self.wh.1,
                self.dc,
                self.top_left.0,
                self.top_left.1,
                Gdi::SRCCOPY,
            )?;

            let slice = std::slice::from_raw_parts(bits, (self.wh.0 * self.wh.1 * 3) as usize);

            Ok(CaptureData {
                bits: slice,
                hbitmap,
            })
        }
    }
    /// Modifies information associated with the screenshot size and position without the need to call the constructor again.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::error::Error;
    /// use qshot::CaptureManager;
    ///
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     let mut manager = CaptureManager::new(0, (250, 250), (500, 500))?;
    ///
    ///     let res = manager.capture()?;
    ///     assert_eq!(res.get_bits().len(), 500 * 500 * 3);
    ///     
    ///     manager.change_size((100, 100), (100, 250));
    ///     
    ///     let res1 = manager.capture()?;
    ///     assert_eq!(res1.get_bits().len(), 100 * 250 * 3);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn change_size(&mut self, top_left: (i32, i32), wh: (i32, i32)) {
        self.wh = wh;
        self.top_left = top_left;
        self.bitmap_info.bmiHeader.biHeight = wh.0;
        self.bitmap_info.bmiHeader.biHeight = -wh.1;
    }
}

impl Drop for CaptureManager {
    fn drop(&mut self) {
        unsafe {
            Gdi::ReleaseDC(self.window_handle, self.dc);
            Gdi::DeleteDC(self.dc_mem);
        }
    }
}
