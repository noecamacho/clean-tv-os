package app.lume.tv;

import android.accessibilityservice.AccessibilityService;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.os.SystemClock;
import android.view.KeyEvent;
import android.view.accessibility.AccessibilityEvent;

/**
 * Makes HOME the launcher without touching the system one: Google TV ignores the preferred home
 * activity on this TCL, so while this service is enabled the Home key and screen-on open HOME.
 * Disabling the service (Settings > Accessibility, or adb) gives the Home key back to Google TV.
 */
public class HomeKeyService extends AccessibilityService {

    private final BroadcastReceiver screenOn = new BroadcastReceiver() {
        @Override
        public void onReceive(Context context, Intent intent) {
            openHome(false);
        }
    };

    @Override
    protected void onServiceConnected() {
        registerReceiver(screenOn, new IntentFilter(Intent.ACTION_SCREEN_ON));
        // Only right after boot: a later reconnect (app update, process restart) must not pull
        // the user out of what they are watching.
        if (SystemClock.elapsedRealtime() < 120_000) {
            openHome(false);
        }
    }

    @Override
    protected boolean onKeyEvent(KeyEvent event) {
        if (event.getKeyCode() != KeyEvent.KEYCODE_HOME) {
            return false;
        }
        if (event.getAction() == KeyEvent.ACTION_UP) {
            openHome(true);
        }
        return true;
    }

    /** `key`: a Home press, which HOME itself also acts on (back to the start, Control Center). */
    private void openHome(boolean key) {
        Intent intent = new Intent(this, LumeActivity.class).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
        if (key) {
            // MAIN/HOME, as the system's own Home press: the activity knows it by the category.
            intent.setAction(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_HOME);
        }
        startActivity(intent);
    }

    @Override
    public void onDestroy() {
        try {
            unregisterReceiver(screenOn);
        } catch (IllegalArgumentException ignored) {
            // Never connected, so never registered.
        }
        super.onDestroy();
    }

    @Override
    public void onAccessibilityEvent(AccessibilityEvent event) {}

    @Override
    public void onInterrupt() {}
}
