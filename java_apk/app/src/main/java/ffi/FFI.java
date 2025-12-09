package ffi;

import android.content.Context;
import java.nio.ByteBuffer;

public class FFI {
    static {
        System.loadLibrary("rustdesk");
    }

    public static native void init(Context ctx);
    public static native void onAppStart(Context ctx);
    // public static native void setClipboardManager(RdClipboardManager clipboardManager); // Skipping for now as it requires extra class
    public static native void startServer(String app_dir, String custom_client_config);
    public static native void startService();
    public static native void onVideoFrameUpdate(ByteBuffer buf);
    public static native void onAudioFrameUpdate(ByteBuffer buf);
    public static native String translateLocale(String localeName, String input);
    public static native void refreshScreen();
    public static native void setFrameRawEnable(String name, boolean value);
    public static native void setCodecInfo(String info);
    public static native String getLocalOption(String key);
    public static native void onClipboardUpdate(ByteBuffer clips);
    public static native boolean isServiceClipboardEnabled();
}
