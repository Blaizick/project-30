#[macro_export]
macro_rules! gl_check {
    () => {{
        let err = unsafe { gl::GetError() };
        assert_eq!(err, gl::NO_ERROR, "GL error: {}", err);
    }};
}

#[macro_export]
macro_rules! gl_call {
    ($call:expr) => {{
        unsafe {
            gl::GetError();
            $call;
            let err = gl::GetError();

            if err != gl::NO_ERROR {
                panic!(
                    "OpenGL error {} at {}:{}\nCall: {}",
                    err,
                    file!(),
                    line!(),
                    stringify!($call)
                );
            }
        }
    }};
}