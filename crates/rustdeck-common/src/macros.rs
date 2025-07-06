#[macro_export]
macro_rules! decorate_fn_init {
    ( $user_fn_init:expr ) => {{
        unsafe extern "C-unwind" fn _fn_init() -> $crate::Result {
            $crate::Result::from(($user_fn_init)())
        }

        _fn_init
    }};
}

#[macro_export]
macro_rules! decorate_fn_update {
    ( $user_fn_update:expr ) => {{
        unsafe extern "C-unwind" fn _fn_update(state: *mut ::std::ffi::c_void) -> $crate::Result {
            let state = unsafe { &mut *state.cast() };
            $crate::Result::from(($user_fn_update)(state))
        }

        _fn_update
    }};
}

#[macro_export]
macro_rules! decorate_fn_get_variable {
    ( $user_fn_get_variable:expr ) => {{
        unsafe extern "C-unwind" fn _fn_get_variable(
            state: *mut ::std::ffi::c_void,
            id: *const ::std::ffi::c_char,
        ) -> $crate::Result {
            let state = unsafe { &mut *state.cast() };
            let id = unsafe { $crate::util::ptr_to_str(id) };
            $crate::Result::from(($user_fn_get_variable)(state, id))
        }

        _fn_get_variable
    }};
}

#[macro_export]
macro_rules! decorate_fn_run_action {
    ( $user_fn_run_action:expr ) => {{
        unsafe extern "C-unwind" fn _fn_run_action(
            state: *mut ::std::ffi::c_void,
            id: *const ::std::ffi::c_char,
            args: *const $crate::proto::Arg,
        ) -> $crate::Result {
            let user_state = unsafe { &mut *state.cast() };
            let id = unsafe { $crate::util::ptr_to_str(id) };
            let args = $crate::Args::from(args);
            $crate::Result::from(($user_fn_run_action)(user_state, id, &args))
        }

        _fn_run_action
    }};
}

#[macro_export]
macro_rules! decorate_fn_get_enum {
    ( $user_fn_get_enum:expr ) => {{
        unsafe extern "C-unwind" fn _get_enum(
            state: *mut ::std::ffi::c_void,
            id: *const ::std::ffi::c_char,
        ) -> $crate::Result {
            let state = unsafe { &mut *state.cast() };
            let id = unsafe { $crate::util::ptr_to_str(id) };
            $crate::Result::from(($user_fn_get_enum)(state, id))
        }

        ::std::boxed::Box::into_raw(::std::boxed::Box::new(
            _get_enum as $crate::proto::FnGetEnum,
        ))
        .cast_const()
    }};
}

#[macro_export]
macro_rules! decorate_fn_get_config_value {
    ( $user_fn_get_config_value:expr ) => {{
        unsafe extern "C-unwind" fn _get_config_value(
            state: *mut ::std::ffi::c_void,
            id: *const ::std::ffi::c_char,
        ) -> $crate::Result {
            let state = unsafe { &mut *state.cast() };
            let id = unsafe { $crate::util::ptr_to_str(id) };
            $crate::Result::from(($user_fn_get_config_value)(state, id))
        }

        ::std::boxed::Box::into_raw(::std::boxed::Box::new(
            _get_config_value as $crate::proto::FnGetConfigValue,
        ))
        .cast_const()
    }};
}

#[macro_export]
macro_rules! decorate_fn_set_config_value {
    ( $user_fn_set_config_value:expr ) => {{
        unsafe extern "C-unwind" fn _set_config_value(
            state: *mut ::std::ffi::c_void,
            id: *const ::std::ffi::c_char,
            value: *const $crate::proto::Arg,
        ) -> $crate::Result {
            let state = unsafe { &mut *state.cast() };
            let id = unsafe { $crate::util::ptr_to_str(id) };
            let value = $crate::Args::from(value);
            $crate::Result::from(($user_fn_set_config_value)(state, id, &value))
        }

        ::std::boxed::Box::into_raw(::std::boxed::Box::new(
            _set_config_value as $crate::proto::FnSetConfigValue,
        ))
        .cast_const()
    }};
}

#[macro_export]
macro_rules! export_plugin {
    ( $in:expr ) => {
        #[unsafe(no_mangle)]
        unsafe extern "C" fn build() -> *const $crate::proto::Plugin {
            $in
        }

        #[unsafe(no_mangle)]
        unsafe extern "C" fn free(ptr: *mut ::std::ffi::c_char) {
            _ = ::std::ffi::CString::from_raw(ptr);
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
        fn_run_action: $user_fn_run_action:expr,

        __fn_get_enum: $user_fn_get_enum:expr

        $(,)?
    ) => {
        unsafe {
            let fn_init = $crate::decorate_fn_init!($user_fn_init);
            let fn_update = $crate::decorate_fn_update!($user_fn_update);
            let fn_get_variable = $crate::decorate_fn_get_variable!($user_fn_get_variable);
            let fn_run_action = $crate::decorate_fn_run_action!($user_fn_run_action);

            ::std::boxed::Box::into_raw(::std::boxed::Box::new($crate::proto::Plugin {
                id: $crate::util::str_to_ptr($id),
                name: $crate::util::str_to_ptr($name),
                desc: $crate::util::str_to_ptr($desc),

                variables: $variables,
                actions: $actions,
                config_options: ::std::ptr::null(),

                fn_init: fn_init,
                fn_update: fn_update,
                fn_get_variable: fn_get_variable,
                fn_run_action: fn_run_action,

                fn_get_enum: $user_fn_get_enum,
                fn_get_config_value: ::std::ptr::null(),
                fn_set_config_value: ::std::ptr::null(),
            })) as *const $crate::proto::Plugin
        }
    };

    /* Without `fn_get_enum` */
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
        decl_plugin! {
            id: $id,
            name: $name,
            desc: $desc,
            variables: $variables,
            actions: $actions,
            fn_init: $user_fn_init,
            fn_update: $user_fn_update,
            fn_get_variable: $user_fn_get_variable,
            fn_run_action: $user_fn_run_action,
            __fn_get_enum: ::std::ptr::null(),
        }
    };

    /* With `fn_get_enum` */
    (
        id: $id:literal,
        name: $name:literal,
        desc: $desc:literal,
        variables: $variables:expr,
        actions: $actions:expr,

        fn_init: $user_fn_init:expr,
        fn_update: $user_fn_update:expr,
        fn_get_variable: $user_fn_get_variable:expr,
        fn_run_action: $user_fn_run_action:expr,

        fn_get_enum: $user_fn_get_enum:expr

        $(,)?
    ) => {
        let fn_get_enum_p = $crate::decorate_fn_get_enum!($user_fn_get_enum);

        unsafe {
            decl_plugin! {
                id: $id,
                name: $name,
                desc: $desc,
                variables: $variables,
                actions: $actions,
                fn_init: $user_fn_init,
                fn_update: $user_fn_update,
                fn_get_variable: $user_fn_get_variable,
                fn_run_action: $user_fn_run_action,
                __fn_get_enum: fn_get_enum_p,
            }
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
                r#type: $crate::Type::from($vtype).into(),
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
        unsafe {
            ::std::boxed::Box::into_raw(::std::boxed::Box::new($crate::proto::Action {
                id: $crate::util::str_to_ptr($id),
                name: $crate::util::str_to_ptr($name),
                desc: $crate::util::str_to_ptr($desc),
                args: $args,
            })) as *const $crate::proto::Action
        }
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
        id: $id:literal,
        name: $name:literal,
        desc: $desc:literal,
        vtype: $vtype:literal
        $(,)?
    ) => {
        unsafe {
            ::std::boxed::Box::into_raw(::std::boxed::Box::new($crate::proto::ActionArg {
                id: $crate::util::str_to_ptr($id),
                name: $crate::util::str_to_ptr($name),
                desc: $crate::util::str_to_ptr($desc),
                r#type: $crate::Type::from($vtype).into(),
            })) as *const $crate::proto::ActionArg
        }
    };
}
