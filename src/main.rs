extern crate winapi;
extern crate user32;
extern crate kernel32;

use winapi::*;
use user32::*;
use kernel32::*;

use std::ptr::{null_mut, null};
use std::mem::{size_of, uninitialized, transmute};

///////
// CONST
///////

pub const MSG_CHECK: UINT =  0x0400;  //WPARAM = 0 | LPARAM = 0


///////
// EXTERNS
///////

#[cfg(any(target_arch = "x86_64"))]
extern "system" {
	pub fn D2D1CreateFactory(
        factoryType: D2D1_FACTORY_TYPE,
		riid: REFIID, 
		pFactoryOptions: *const D2D1_FACTORY_OPTIONS,
        ppIFactory: *mut *mut ID2D1Factory
    ) -> HRESULT;
}

///////
// STRUCTURES
///////

pub struct MyAppRessources{
    render_target: *mut ID2D1HwndRenderTarget,
    brush1: *mut ID2D1SolidColorBrush,
    brush2: *mut ID2D1SolidColorBrush
}

pub struct MyApp{
    ressources: Option<MyAppRessources>,
    factory: Option<*mut ID2D1Factory>,
    hwnd: Option<HWND>,
    ok: i32
}


///////
// D2D1 SETUP
///////

/*
    Create a D2D1CreateFactory
*/
unsafe fn setup_d2d_factory(app: &mut MyApp){
    let null_options: *const D2D1_FACTORY_OPTIONS = null();
    let mut factory: *mut ID2D1Factory = null_mut();
    
    let result = D2D1CreateFactory(
        D2D1_FACTORY_TYPE_SINGLE_THREADED,
        &UuidOfID2D1Factory,
        null_options,
        &mut factory
    );
    
    if result != S_OK{
        panic!("Could not create D2D1 factory.");
    }
    
   app.factory = Some(factory)
} 

/*
    Create the ressource used when drawing in the window.
    
*/
unsafe fn setup_d2d_ressources(app: &mut MyApp){    
    
    //Check if the ressources are allocated.
    if app.ressources.is_some(){
        return;
    }
    
    let hwnd = app.hwnd.unwrap();
    let size: D2D_SIZE_U;
	let mut rc: RECT = uninitialized();
    
    let mut ressources = MyAppRessources{
        render_target: null_mut(),
        brush1: null_mut(),
        brush2: null_mut(),
    };
    
    /*
        Structures for CreateHwndRenderTarget
    */
    GetClientRect(hwnd, &mut rc);
    size = D2D_SIZE_U{width: (rc.right-rc.left) as u32,
				      height: (rc.bottom-rc.top) as u32};
    
    let pixel_format = D2D1_PIXEL_FORMAT{
        format: DXGI_FORMAT_B8G8R8A8_UNORM.0,
        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED
    };
    
    let render_props = D2D1_RENDER_TARGET_PROPERTIES{
        _type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
        pixelFormat: pixel_format,
        dpiX: 0.0, dpiY: 0.0,
        usage: D2D1_RENDER_TARGET_USAGE_NONE,
        minLevel: D2D1_FEATURE_LEVEL_DEFAULT
    };
    
    let hwnd_render_props = D2D1_HWND_RENDER_TARGET_PROPERTIES{
        hwnd: hwnd,
        pixelSize: size,
        presentOptions: D2D1_PRESENT_OPTIONS_NONE
    };
    
    /*
        Structures for ID2D1SolidColorBrush
    */
    let null_properties: *const D2D1_BRUSH_PROPERTIES = null();
    let gray = D2D1_COLOR_F{r: 0.745, g: 0.823, b: 0.863, a: 1.0};
    let red = D2D1_COLOR_F{r: 0.941, g: 0.353, b: 0.392, a: 1.0};
    
    /*
        Allocate the ressources
    */
    match app.factory{
        Some(f) => {
            let factory: &mut ID2D1Factory = transmute(f);
            let mut rt: &mut ID2D1HwndRenderTarget;
            
            if factory.CreateHwndRenderTarget(&render_props, &hwnd_render_props, &mut ressources.render_target) != S_OK{
                panic!("Could not create render target.");
            }
            
            rt = transmute(ressources.render_target);
            
            if rt.CreateSolidColorBrush(&gray, null_properties, &mut ressources.brush1) != S_OK{
				panic!("Could not create brush!");
			}
            
            if rt.CreateSolidColorBrush(&red, null_properties, &mut ressources.brush2) != S_OK{
				panic!("Could not create brush!");
			}
            
        },
        None => panic!("What?")
    }
    
    app.ressources = Some(ressources);
}


/*
    Release the ressources used by Direct2D
*/
unsafe fn clean_d2d_ressources(app: &mut MyApp){
    match app.ressources.as_mut(){
        Some(r) => {
            transmute::<_, &mut IUnknown>(r.brush1).Release();
            transmute::<_, &mut IUnknown>(r.brush2).Release();
            transmute::<_, &mut IUnknown>(r.render_target).Release();
            
            r.brush1 = null_mut();
            r.brush2 = null_mut();
            r.render_target = null_mut();
        },
        None => {}
    }

    app.ressources = None;
}

/*
    Release the ressources used by Direct2D
*/
unsafe fn clean_d2d(app: &mut MyApp){
    clean_d2d_ressources(app);
    
    match app.factory{
        Some(i) => {
            transmute::<_, &mut IUnknown>(i).Release();
            app.factory = None;
        },
        None => panic!("What?")
    }
}

///////
// WINDOW PROCEDURE
///////

unsafe extern "system" fn wndproc(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT{
    let mut result: (LPARAM, bool) = (1, true);
    let myapp: &mut MyApp = transmute(GetWindowLongPtrW(hwnd, 0));
   
    match msg{
        WM_PAINT =>{
            let mut result = S_OK;
            match myapp.ressources.as_mut(){
                Some(r) => {
                    let render: &mut ID2D1HwndRenderTarget = transmute(r.render_target);
                    render.BeginDraw();
                    
                    result = render.EndDraw(null_mut(), null_mut());
                }
                None => {}
            }
            
            // Check if the ressources needs to be recreated.
            if result == D2DERR_RECREATE_TARGET{
                clean_d2d_ressources(myapp);
            }
        }
        MSG_CHECK =>{
            // Check if the application data in in the window data.
            if myapp.ok != 322{
                panic!("MyApp is not in the window!");
            }else{
                println!("\nEverything is fine!");
            }
        },
        WM_DESTROY =>{
            PostQuitMessage(0);
        },
        WM_CREATE => {
        },
        _ => {result = (0, false);}
    }

    match result.1{
        true => result.0,
        false => DefWindowProcW(hwnd, msg, w, l)
    }
}

///////
// WINDOW SETUP
///////

/*
    Create the window class.
*/
unsafe fn setup_class(class_name: &Vec<WCHAR>){
    let null_icon: HICON = null_mut();
    let null_background: HBRUSH = null_mut();
    let null_name: *const WCHAR = null();
    let module = GetModuleHandleW(null_name);
    
    let class =
    WNDCLASSEXW{
			cbSize: size_of::<WNDCLASSEXW>() as UINT,
			style: CS_HREDRAW | CS_VREDRAW,
			lpfnWndProc: Some(wndproc), 
			cbClsExtra: 0,
			cbWndExtra: 32,
			hInstance: module,
			hIcon: null_icon,
			hCursor: LoadCursorW(module, IDC_ARROW),
			hbrBackground: null_background,
			lpszMenuName: null_name,
			lpszClassName:  class_name.as_ptr(),
			hIconSm: null_icon
	};
    
    //Register the class
    match RegisterClassExW(&class){
        0 => panic!("Could not register class!"),
        _ => {}
    };
}

/*
    Create the window
*/
unsafe fn setup_window(app: &mut MyApp, class_name: &Vec<WCHAR>, window_name: &Vec<WCHAR>){
    let null_hwnd: HWND = null_mut();
    let null_menu: HMENU = null_mut();
    let null_name: *const WCHAR = null();
    let null: LPVOID = null_mut();
    let module = GetModuleHandleW(null_name);
    
    let hwnd = 
    CreateWindowExW(
        WS_EX_COMPOSITED,
        class_name.as_ptr(),
        window_name.as_ptr(),
        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        CW_USEDEFAULT, CW_USEDEFAULT,
        600, 400,
        null_hwnd,
        null_menu,
        module,
        null
    );
    
    if hwnd.is_null(){
        panic!("Could not create window");
    }
    
    app.hwnd = Some(hwnd);
}

/*
    Save the app address inside the window data.
    Send a custom message to ensure everything is fine.
*/
unsafe fn pack_app(app: &mut MyApp){
    match app.hwnd{
        Some(hwnd) => {
            SetWindowLongPtrW(hwnd, 0, transmute(app));
            PostMessageW(hwnd, MSG_CHECK, 0, 0);
        },
        None => panic!("What?")
    }
}



///////
// MAIN
///////

fn main() {
    unsafe{
        let mut app = MyApp{ok: 322, ressources: None, factory: None, hwnd: None};
        
        // 'MyApp' as UTF16
        let class_name: Vec<WCHAR> = vec![77, 121, 65, 112, 112, 0];
        let window_name: Vec<WCHAR> = vec![77, 121, 65, 112, 112, 0];
       
        // Window setup
        setup_class(&class_name);
        setup_window(&mut app, &class_name, &window_name);
        pack_app(&mut app);
        
        // D2D1 Setup
        setup_d2d_factory(&mut app);
        setup_d2d_ressources(&mut app);
        
        // Application Loop
        let mut msg = uninitialized();
		let null_handle: HWND = null_mut();
        while GetMessageW(&mut msg, null_handle, 0, 0) != 0{
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        
        //App cleaning
        clean_d2d(&mut app);
    }
    
}
