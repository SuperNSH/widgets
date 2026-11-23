//! Android: the widget runs in the app's own process, so the store is in the app's files directory;
//! `dev.supernsh.widgets.NshWidgets.update` redraws a kind's placed widgets.

use jni::{
    JavaVM,
    objects::{JClass, JObject, JString, JValue},
};

use crate::WidgetError;

fn with_env<R>(f: impl FnOnce(&mut jni::JNIEnv, &JObject) -> jni::errors::Result<R>) -> Result<R, WidgetError> {
    let ctx = ndk_context::android_context();
    // SAFETY: ndk-context hands out the process's JavaVM and the app context NSH registered.
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }.map_err(|e| WidgetError::Io(e.to_string()))?;
    let mut env = vm.attach_current_thread().map_err(|e| WidgetError::Io(e.to_string()))?;
    // SAFETY: the context stays alive for the app's lifetime.
    let context = unsafe { JObject::from_raw(ctx.context().cast()) };
    let out = f(&mut env, &context);
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
    }
    out.map_err(|e| WidgetError::Io(e.to_string()))
}

pub(crate) fn root(_group: Option<String>) -> Result<String, WidgetError> {
    with_env(|env, ctx| {
        let dir = env.call_method(ctx, "getFilesDir", "()Ljava/io/File;", &[])?.l()?;
        let path = env
            .call_method(&dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?
            .l()?;
        Ok(env.get_string(&JString::from(path))?.into())
    })
}

pub(crate) fn reload(kind: &str) -> Result<(), WidgetError> {
    with_env(|env, ctx| {
        // App classes need the app's class loader: FindClass on a native thread sees only the system's.
        let loader = env
            .call_method(ctx, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?
            .l()?;
        let name = env.new_string("dev.supernsh.widgets.NshWidgets")?;
        let class = env
            .call_method(
                &loader,
                "loadClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
                &[JValue::Object(&name)],
            )?
            .l()?;
        let kind = env.new_string(kind)?;
        env.call_static_method(
            JClass::from(class),
            "update",
            "(Landroid/content/Context;Ljava/lang/String;)V",
            &[JValue::Object(ctx), JValue::Object(&kind)],
        )?;
        Ok(())
    })
}

/// The receiver rewrites data.json in this process, which the store's change token sees.
pub(crate) fn observe(_group: Option<String>) {}

pub(crate) fn change_count() -> i64 {
    0
}

/// Live Activities exist only on iOS.
pub(crate) fn activity_start(_kind: &str, _data: &str) -> Result<(), WidgetError> {
    Err(WidgetError::Unsupported)
}

pub(crate) fn activity_update(_kind: &str, _data: &str) -> Result<(), WidgetError> {
    Err(WidgetError::Unsupported)
}

pub(crate) fn activity_end(_kind: &str) -> Result<(), WidgetError> {
    Err(WidgetError::Unsupported)
}
