package com.example.qrcodereceiver

class DecoderCore {
    private var decoderPtr: Long = 0

    init {
        // Load the Rust JNI library
        System.loadLibrary("shared_core")
        decoderPtr = initDecoder()
    }

    /**
     * Processes a single QR code payload frame.
     * @return The decoded file bytes if the file is completely reconstructed, otherwise null or empty.
     */
    fun processFrame(frameData: ByteArray): ByteArray? {
        if (decoderPtr == 0L) return null
        val result = processFrame(decoderPtr, frameData)
        return if (result.isNotEmpty()) result else null
    }

    /**
     * Gets the current progress of the transfer (0.0 to 1.0).
     */
    fun getProgress(): Float {
        if (decoderPtr == 0L) return 0.0f
        return getProgress(decoderPtr)
    }

    fun destroy() {
        if (decoderPtr != 0L) {
            destroyDecoder(decoderPtr)
            decoderPtr = 0L
        }
    }

    protected finalize() {
        destroy()
    }

    // Native methods mapping to Rust JNI
    private external fun initDecoder(): Long
    private external fun processFrame(ptr: Long, frameData: ByteArray): ByteArray
    private external fun getProgress(ptr: Long): Float
    private external fun destroyDecoder(ptr: Long)
}
