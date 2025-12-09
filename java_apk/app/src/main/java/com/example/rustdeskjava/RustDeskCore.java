package com.example.rustdeskjava;

import java.nio.ByteBuffer;

public class RustDeskCore {
    static {
        System.loadLibrary("rustdesk");
    }

    public static native void init();
    public static native void startSession(String id, String password);
    public static native void stopSession(String id);
    public static native void getRgba(String id, int display, ByteBuffer buffer);
    public static native int getRgbaSize(String id);
    public static native int getWidth(String id);
    public static native int getHeight(String id);
}
