package app.lume.tv;

import android.app.NativeActivity;
import android.content.Intent;

/**
 * Thin activity: all logic lives in liblume.so (Rust). It exposes the bridge to the framework
 * and tells the native side when the Home button is pressed while lume is already showing.
 */
public class LumeActivity extends NativeActivity {
    static {
        // Needed for the native method below. A failure only disables that one feature.
        try {
            System.loadLibrary("lume");
        } catch (UnsatisfiedLinkError e) {
            // Ignore: the activity still works without the Home-button notification.
        }
    }

    /** Called by Rust through JNI: runs one bridge command. */
    public String bridge(String command, String argument) {
        return LumeBridge.handle(this, command, argument);
    }

    /** Called by Rust through JNI: what a tile shows for an app: see {@link LumeBridge#iconPixels}. */
    public byte[] iconPixels(String packageName, String activityName, int width, int height) {
        return LumeBridge.iconPixels(this, packageName, activityName, width, height);
    }

    /** Called by Rust through JNI: the square icon of an app for the dock: see {@link LumeBridge#dockIcon}. */
    public byte[] dockIcon(String packageName, String activityName, int size) {
        return LumeBridge.dockIcon(this, packageName, activityName, size);
    }

    /** Implemented in Rust (crates/lume-app/src/android/mod.rs). */
    private static native void nativeHomePressed();

    @Override
    protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        if (intent != null && intent.hasCategory(Intent.CATEGORY_HOME)) {
            try {
                nativeHomePressed();
            } catch (UnsatisfiedLinkError e) {
                // The native side is not available: nothing to notify.
            }
        }
    }
}
