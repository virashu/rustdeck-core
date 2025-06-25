#[macro_export]
macro_rules! export_plugin {
    ( $in:expr ) => {
        #[unsafe(no_mangle)]
        unsafe extern "C" fn build() -> *const $crate::proto::Plugin {
            $in
        }
    };
}

#[macro_export]
macro_rules! decl_plugin {
    /* With actions and variables */
    (
        id: $id:literal,
        name: $name:literal,
        desc: $desc:literal,
        variables: $variables:expr,
        actions: $actions:expr,

        fn_init: $user_fn_init:expr,
        fn_update: $user_fn_update:expr,
        fn_get_variable: $user_fn_get_variable:expr,
        fn_run_action: $user_fn_run_action:expr

        $(,)?
    ) => {
        unsafe {
            unsafe extern "C" fn fn_init() -> *mut ::std::ffi::c_void {
                let mut user_state = ($user_fn_init)();
                let mut state = ::std::mem::ManuallyDrop::new(Box::new(user_state));
                (&raw mut (**state)).cast()
            }
            unsafe extern "C" fn fn_update(state: *mut ::std::ffi::c_void) {
                let user_state = unsafe { &mut *state.cast() };
                ($user_fn_update)(user_state);
            }
            unsafe extern "C" fn fn_get_variable(
                state: *mut ::std::ffi::c_void,
                id: *const ::std::ffi::c_char,
            ) -> *mut ::std::ffi::c_char {
                let user_state = unsafe { &mut *state.cast() };
                let id = unsafe { ::std::ffi::CStr::from_ptr(id).to_str().unwrap() };
                let res = ($user_fn_get_variable)(user_state, id);

                return (*::std::mem::ManuallyDrop::new(::std::boxed::Box::new(
                    ::std::ffi::CString::new(res).unwrap(),
                )))
                .as_ptr()
                .cast_mut();
            }
            unsafe extern "C" fn fn_run_action(
                state: *mut ::std::ffi::c_void,
                id: *const ::std::ffi::c_char,
                args: *const $crate::proto::Arg,
            ) {
                let user_state = unsafe { &mut *state.cast() };
                let id = unsafe { ::std::ffi::CStr::from_ptr(id).to_str().unwrap() };
                ($user_fn_run_action)(user_state, id, args);
            }

            ::std::boxed::Box::into_raw(::std::boxed::Box::new($crate::proto::Plugin {
                id: $crate::util::str_to_ptr($id),
                name: $crate::util::str_to_ptr($name),
                desc: $crate::util::str_to_ptr($desc),
                variables: $variables,
                actions: $actions,

                fn_init: fn_init,
                fn_update: fn_update,
                fn_get_variable: fn_get_variable,
                fn_run_action: fn_run_action,
            })) as *const $crate::proto::Plugin
        }
    };

    /* Without actions nor variables */
    (
        id: $id:literal,
        name: $name:literal,
        desc: $desc:literal,

        fn_init: $user_fn_init:expr,
        fn_update: $user_fn_update:expr,
        fn_get_variable: $user_fn_get_variable:expr,
        fn_run_action: $user_fn_run_action:expr

        $(,)?
    ) => {
        decl_plugin! {
            id: $id,
            name: $name,
            desc: $desc,
            variables: ::std::ptr::null(),
            actions: ::std::ptr::null(),
            fn_init: $user_fn_init,
            fn_update: $user_fn_update,
            fn_get_variable: $user_fn_get_variable,
            fn_run_action: $user_fn_run_action
        }
    };

    /* With variables */
    (
        id: $id:literal,
        name: $name:literal,
        desc: $desc:literal,
        variables: $variables:expr,

        fn_init: $user_fn_init:expr,
        fn_update: $user_fn_update:expr,
        fn_get_variable: $user_fn_get_variable:expr,
        fn_run_action: $user_fn_run_action:expr

        $(,)?
    ) => {
        decl_plugin! {
            id: $id,
            name: $name,
            desc: $desc,
            variables: $variables,
            actions: ::std::ptr::null(),
            fn_init: $user_fn_init,
            fn_update: $user_fn_update,
            fn_get_variable: $user_fn_get_variable,
            fn_run_action: $user_fn_run_action
        }
    };

    /* With actions */
    (
        id: $id:literal,
        name: $name:literal,
        desc: $desc:literal,
        actions: $actions:expr,

        fn_init: $user_fn_init:expr,
        fn_update: $user_fn_update:expr,
        fn_get_variable: $user_fn_get_variable:expr,
        fn_run_action: $user_fn_run_action:expr

        $(,)?
    ) => {
        decl_plugin! {
            id: $id,
            name: $name,
            desc: $desc,
            variables: ::std::ptr::null(),
            actions: $actions,
            fn_init: $user_fn_init,
            fn_update: $user_fn_update,
            fn_get_variable: $user_fn_get_variable,
            fn_run_action: $user_fn_run_action
        }
    };
}

#[macro_export]
macro_rules! variables {
    (
        $($var:expr),+ $(,)?
    ) => {
        unsafe {
            ::std::mem::ManuallyDrop::new(vec![
                $($var,)+
                ::std::ptr::null()
            ]).as_ptr() as *const *const $crate::proto::Variable
        }
    };
}

#[macro_export]
macro_rules! decl_variable {
    (
        id: $id:literal,
        desc: $desc:literal,
        vtype: $vtype:literal
        $(,)?
    ) => {
        unsafe {
            ::std::boxed::Box::into_raw(::std::boxed::Box::new($crate::proto::Variable {
                id: $crate::util::str_to_ptr($id),
                desc: $crate::util::str_to_ptr($desc),
                r#type: $crate::Type::try_from($vtype)
                    .expect("Incorrect variable type")
                    .into(),
            })) as *const $crate::proto::Variable
        }
    };
}

#[macro_export]
macro_rules! actions {
    (
        $($act:expr),+ $(,)?
    ) => {
        unsafe {
            ::std::mem::ManuallyDrop::new(vec![
                $($act,)+
                ::std::ptr::null()
            ]).as_ptr() as *const *const $crate::proto::Action
        }
    };
}

#[macro_export]
macro_rules! decl_action {
    (
        id: $id:literal,
        name: $name:literal,
        desc: $desc:literal,
        args: $args:expr
        $(,)?
    ) => {
        ::std::boxed::Box::into_raw(::std::boxed::Box::new($crate::proto::Action {
            id: $crate::util::str_to_ptr($id),
            name: $crate::util::str_to_ptr($name),
            desc: $crate::util::str_to_ptr($desc),
            args: $args,
        })) as *const $crate::proto::Action
    };

    (
        id: $id:literal,
        name: $name:literal,
        desc: $desc:literal
        $(,)?
    ) => {
        decl_action! {
            id: $id,
            name: $name,
            desc: $desc,
            args: ::std::ptr::null(),
        }
    };
}

#[macro_export]
macro_rules! args {
    (
        $($arg:expr),+ $(,)?
    ) => {
        unsafe {
            ::std::mem::ManuallyDrop::new(vec![
                $($arg,)+
                ::std::ptr::null()
            ]).as_ptr() as *const *const $crate::proto::ActionArg
        }
    };
}

#[macro_export]
macro_rules! decl_arg {
    (
        name: $name:literal,
        desc: $desc:literal,
        vtype: $vtype:literal
        $(,)?
    ) => {
        ::std::boxed::Box::into_raw(::std::boxed::Box::new($crate::proto::ActionArg {
            name: $crate::util::str_to_ptr($name),
            desc: $crate::util::str_to_ptr($desc),
            r#type: $crate::Type::try_from($vtype)
                .expect("Incorrect variable type")
                .into(),
        })) as *const $crate::proto::ActionArg
    };
}
