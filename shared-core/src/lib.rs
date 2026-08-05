pub mod protocol;
pub mod encoder;
pub mod decoder;

// JNI bindings for Android
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
pub mod jni_exports {
    use crate::decoder::QrDecoder;
    use jni::objects::{JClass, JByteArray};
    use jni::sys::{jlong, jfloat};
    use jni::JNIEnv;

    #[no_mangle]
    pub extern "system" fn Java_com_example_qrcodereceiver_DecoderCore_initDecoder(
        _env: JNIEnv,
        _class: JClass,
    ) -> jlong {
        let decoder = Box::new(QrDecoder::new());
        Box::into_raw(decoder) as jlong
    }

    #[no_mangle]
    pub extern "system" fn Java_com_example_qrcodereceiver_DecoderCore_processFrame<'local>(
        mut env: JNIEnv<'local>,
        _class: JClass,
        decoder_ptr: jlong,
        frame_data: JByteArray<'local>,
    ) -> JByteArray<'local> {
        let decoder = unsafe { &mut *(decoder_ptr as *mut QrDecoder) };
        let bytes = env.convert_byte_array(&frame_data).unwrap_or_default();
        
        if let Some(decoded_file) = decoder.process_frame(&bytes) {
            env.byte_array_from_slice(&decoded_file).unwrap_or(JByteArray::default())
        } else {
            JByteArray::default()
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_example_qrcodereceiver_DecoderCore_getProgress(
        _env: JNIEnv,
        _class: JClass,
        decoder_ptr: jlong,
    ) -> jfloat {
        let decoder = unsafe { &mut *(decoder_ptr as *mut QrDecoder) };
        decoder.progress() as jfloat
    }

    #[no_mangle]
    pub extern "system" fn Java_com_example_qrcodereceiver_DecoderCore_destroyDecoder(
        _env: JNIEnv,
        _class: JClass,
        decoder_ptr: jlong,
    ) {
        if decoder_ptr != 0 {
            unsafe {
                let _ = Box::from_raw(decoder_ptr as *mut QrDecoder);
            }
        }
    }
}
