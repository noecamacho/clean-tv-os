package app.lume.tv;

import android.app.ActivityManager;
import android.content.ActivityNotFoundException;
import android.content.ComponentName;
import android.content.Context;
import android.content.Intent;
import android.content.pm.ActivityInfo;
import android.content.pm.ApplicationInfo;
import android.content.pm.ChangedPackages;
import android.content.pm.PackageManager;
import android.content.pm.ResolveInfo;
import android.content.res.Resources;
import android.graphics.Bitmap;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.graphics.PaintFlagsDrawFilter;
import android.graphics.drawable.AdaptiveIconDrawable;
import android.graphics.drawable.Drawable;
import android.net.Uri;
import android.os.Build;
import android.os.Environment;
import android.os.LocaleList;
import android.os.StatFs;
import android.provider.Settings;
import android.text.format.DateFormat;
import android.util.DisplayMetrics;
import android.util.Log;
import android.view.WindowManager;
import java.io.BufferedReader;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.FileReader;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.TimeZone;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Everything the Rust side needs from the Android framework, behind one text command.
 *
 * Rust calls {@link #handle} through {@link LumeActivity#bridge}. A command and its argument
 * are strings, and so is the answer: lists have one record per line and fields are separated by
 * tabs (see crates/lume-core/src/shell/protocol.rs for the formats).
 */
public final class LumeBridge {
    /** Side of the icon bitmap handed to Rust, in pixels. */
    private static final int ICON_SIZE = 24;
    /** Side of the icon in a tile, over the tile height. */
    private static final float ICON_RATIO = 0.5f;
    /** The dock icon's corner radius over its side: a rounded square. */
    /** An adaptive icon layer is 108 dp wide and its visible area, the part a mask keeps, 72 dp. */
    private static final float ADAPTIVE_LAYER_RATIO = 108f / 72f;
    /** Reused by every call of {@link #dockIcon}, like the tile bitmaps below. */
    private static Bitmap dockBitmap;
    /** Reused by every call of {@link #iconPixels}: the cell. */
    private static Bitmap cellBitmap;
    /** Folder, inside the cache directory, of the saved tile images. */
    private static final String TILE_CACHE_DIR = "tiles";
    /** Changes when the drawing of a tile changes, so old files are not reused. */
    private static final String TILE_CACHE_VERSION = "v2";
    /** Same for the dock icons, which are drawn by other code. */
    private static final String DOCK_CACHE_VERSION = "d5";
    private static boolean cachePruned;

    /** Sequence number of the last package change report; -1 before the first one. */
    private static int changeSequence = -1;
    /** Where the sequence is kept between runs, with the boot count it belongs to. */
    private static final String SEQUENCE_FILE = "change-sequence.v1";

    private LumeBridge() {}

    public static String handle(Context context, String command, String argument) {
        try {
            switch (command) {
                case "package":
                    return context.getPackageName();
                case "apps":
                    return listApps(context, null);
                case "query":
                    return listApps(context, argument);
                case "icon":
                    return icon(context, argument);
                case "changed":
                    return changed(context);
                case "start":
                    return start(context, argument);
                case "settings":
                    return openSettings(context);
                case "inputs":
                    return inputsAvailable(context);
                case "open-inputs":
                    return openInputs(context);
                case "open-setting":
                    return openSetting(context, argument);
                case "app-info":
                    return openAppInfo(context, argument);
                case "uninstall":
                    return uninstall(context, argument);
                case "stats":
                    return stats(context);
                case "locales":
                    return LocaleList.getDefault().toLanguageTags();
                case "clock":
                    return clock(context);
                case "display":
                    return display(context);
                default:
                    return "";
            }
        } catch (RuntimeException e) {
            return "";
        }
    }

    /**
     * `package TAB label TAB activity TAB 1|0 TAB 1|0` per launchable activity: the first flag is 1
     * for a TV launcher entry, the second 1 for a system app (which cannot be uninstalled).
     */
    private static String listApps(Context context, String onlyPackage) {
        PackageManager pm = context.getPackageManager();
        StringBuilder out = new StringBuilder();
        append(out, pm, launcherIntent(Intent.CATEGORY_LEANBACK_LAUNCHER, onlyPackage), true);
        append(out, pm, launcherIntent(Intent.CATEGORY_LAUNCHER, onlyPackage), false);
        return out.toString();
    }

    private static Intent launcherIntent(String category, String onlyPackage) {
        Intent intent = new Intent(Intent.ACTION_MAIN).addCategory(category);
        if (onlyPackage != null) {
            intent.setPackage(onlyPackage);
        }
        return intent;
    }

    private static void append(StringBuilder out, PackageManager pm, Intent intent, boolean leanback) {
        List<ResolveInfo> found = pm.queryIntentActivities(intent, 0);
        for (ResolveInfo info : found) {
            if (info.activityInfo == null) {
                continue;
            }
            // Plain LAUNCHER entries of system apps are services and setup screens (remote service,
            // OTA, factory menus), not apps for the user. The TV launcher shows none of them.
            boolean system = isSystemApp(info);
            if (!leanback && system) {
                continue;
            }
            out.append(info.activityInfo.packageName)
                .append('\t')
                .append(clean(info.loadLabel(pm)))
                .append('\t')
                .append(info.activityInfo.name)
                .append('\t')
                .append(leanback ? '1' : '0')
                .append('\t')
                .append(system ? '1' : '0')
                .append('\n');
        }
    }

    private static boolean isSystemApp(ResolveInfo info) {
        ApplicationInfo app = info.activityInfo.applicationInfo;
        return app != null
            && (app.flags & (ApplicationInfo.FLAG_SYSTEM | ApplicationInfo.FLAG_UPDATED_SYSTEM_APP)) != 0;
    }

    /**
     * What a tile shows for an app, as `width` x `height` premultiplied RGBA bytes plus one last
     * byte: 1 when it is the app's TV banner, which fills the whole tile, and 0 when it is the
     * icon, centered on a transparent background (the tile color shows through). `null` when the
     * app has neither.
     */
    static synchronized byte[] iconPixels(
        Context context, String packageName, String activityName, int width, int height) {
        File cached = cacheFile(context, packageName, activityName, width, height);
        byte[] pixels = cached == null ? null : readCache(cached, width * height * 4 + 1);
        if (pixels != null) {
            return pixels;
        }
        pixels = renderTile(context, packageName, activityName, width, height);
        if (pixels != null && cached != null) {
            writeCache(context, cached, packageName, pixels);
        }
        return pixels;
    }

    /**
     * The icon of an app for the dock, as `size` x `size` premultiplied RGBA bytes: the whole icon
     * (an adaptive icon with both its layers, not cut by the system's mask). `null` when the app
     * has no icon.
     */
    static synchronized byte[] dockIcon(Context context, String packageName, String activityName, int size) {
        File cached = cacheFile(context, packageName, activityName, size, size, DOCK_CACHE_VERSION);
        byte[] pixels = cached == null ? null : readCache(cached, size * size * 4);
        if (pixels != null) {
            return pixels;
        }
        pixels = renderDockIcon(context, packageName, activityName, size);
        if (pixels != null && cached != null) {
            writeCache(context, cached, packageName, pixels);
        }
        return pixels;
    }

    private static byte[] renderDockIcon(Context context, String packageName, String activityName, int size) {
        PackageManager pm = context.getPackageManager();
        Drawable drawable = loadIcon(pm, new ComponentName(packageName, activityName), packageName);
        if (drawable == null) {
            return null;
        }
        if (dockBitmap == null || dockBitmap.getWidth() != size) {
            dockBitmap = Bitmap.createBitmap(size, size, Bitmap.Config.ARGB_8888);
        }
        dockBitmap.eraseColor(Color.TRANSPARENT);
        drawWholeIcon(new Canvas(dockBitmap), drawable, size);
        ByteBuffer buffer = ByteBuffer.allocate(size * size * 4);
        dockBitmap.copyPixelsToBuffer(buffer);
        return buffer.array();
    }

    /**
     * Draws the icon over the whole square. An adaptive icon is drawn layer by layer, each one
     * enlarged so its visible area fills the square: drawing it as a whole would apply the system's
     * mask (a circle on some devices) and leave transparent corners.
     */
    private static void drawWholeIcon(Canvas canvas, Drawable drawable, int size) {
        canvas.setDrawFilter(new PaintFlagsDrawFilter(0, Paint.ANTI_ALIAS_FLAG | Paint.FILTER_BITMAP_FLAG));
        drawable.setFilterBitmap(true);
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O && drawable instanceof AdaptiveIconDrawable) {
            AdaptiveIconDrawable adaptive = (AdaptiveIconDrawable) drawable;
            int layer = Math.round(size * ADAPTIVE_LAYER_RATIO);
            int offset = (layer - size) / 2;
            for (Drawable part : new Drawable[] {adaptive.getBackground(), adaptive.getForeground()}) {
                if (part != null) {
                    part.setFilterBitmap(true);
                    part.setBounds(-offset, -offset, size + offset, size + offset);
                    part.draw(canvas);
                }
            }
            return;
        }
        drawable.setBounds(0, 0, size, size);
        drawable.draw(canvas);
    }

    /** Draws the tile image; this opens the other app's resources, which is what the cache avoids. */
    private static byte[] renderTile(
        Context context, String packageName, String activityName, int width, int height) {
        PackageManager pm = context.getPackageManager();
        ComponentName component = new ComponentName(packageName, activityName);
        // The same bitmap serves every app: a new one for each would leave the native heap
        // full of freed pages the allocator takes a while to give back.
        if (cellBitmap == null || cellBitmap.getWidth() != width || cellBitmap.getHeight() != height) {
            cellBitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888);
        }
        Bitmap bitmap = cellBitmap;
        bitmap.eraseColor(Color.TRANSPARENT);
        boolean banner = false;
        Drawable drawable = loadBanner(pm, component, packageName);
        if (drawable != null) {
            drawBanner(bitmap, drawable);
            banner = true;
        } else {
            drawable = loadIcon(pm, component, packageName);
            if (drawable == null) {
                return null;
            }
            drawIcon(bitmap, drawable);
        }
        ByteBuffer buffer = ByteBuffer.allocate(width * height * 4 + 1);
        bitmap.copyPixelsToBuffer(buffer);
        buffer.put(width * height * 4, (byte) (banner ? 1 : 0));
        return buffer.array();
    }

    /**
     * Where the image of an activity is cached: the name has the app's last update time, so an
     * updated app gets a new file, and `TILE_CACHE_VERSION`, which changes when the drawing does.
     * `null` when the app is not installed.
     */
    private static File cacheFile(
        Context context, String packageName, String activityName, int width, int height) {
        return cacheFile(context, packageName, activityName, width, height, TILE_CACHE_VERSION);
    }

    private static File cacheFile(
        Context context, String packageName, String activityName, int width, int height, String version) {
        try {
            long updated = context.getPackageManager().getPackageInfo(packageName, 0).lastUpdateTime;
            String name = packageName + "-" + updated + "-" + Integer.toHexString(activityName.hashCode())
                + "-" + width + "x" + height + "-" + version + ".rgba";
            return new File(new File(context.getCacheDir(), TILE_CACHE_DIR), name);
        } catch (PackageManager.NameNotFoundException | RuntimeException e) {
            return null;
        }
    }

    /** The cached bytes, or `null` when there is no file or it has the wrong size. */
    private static byte[] readCache(File file, int expectedLength) {
        if (!file.isFile() || file.length() != expectedLength) {
            return null;
        }
        byte[] bytes = new byte[expectedLength];
        try (FileInputStream in = new FileInputStream(file)) {
            int read = 0;
            while (read < bytes.length) {
                int n = in.read(bytes, read, bytes.length - read);
                if (n < 0) {
                    return null;
                }
                read += n;
            }
            return bytes;
        } catch (IOException e) {
            return null;
        }
    }

    /** Saves the image (written aside and renamed, so a crash never leaves half a file). */
    private static void writeCache(Context context, File file, String packageName, byte[] pixels) {
        File dir = file.getParentFile();
        if (dir == null || (!dir.isDirectory() && !dir.mkdirs())) {
            return;
        }
        File temporary = new File(dir, file.getName() + ".tmp");
        try (FileOutputStream out = new FileOutputStream(temporary)) {
            out.write(pixels);
        } catch (IOException e) {
            temporary.delete();
            return;
        }
        if (!temporary.renameTo(file)) {
            temporary.delete();
            return;
        }
        pruneCache(context, dir, packageName, file.getName());
    }

    /**
     * Deletes what is stale: files of this app from an older update, files of the same kind (tile
     * or dock, told apart by their size) that this one replaces and, once per run, the files of
     * apps that are gone. The other kind of the same app, from the same update, stays.
     */
    private static void pruneCache(Context context, File dir, String packageName, String keep) {
        File[] files = dir.listFiles();
        if (files == null) {
            return;
        }
        boolean checkAll = !cachePruned;
        cachePruned = true;
        // Names are package-updated-hash-WxH-version.rgba, and package names have no dash.
        String[] kept = keep.split("-");
        for (File file : files) {
            String name = file.getName();
            if (name.equals(keep)) {
                continue;
            }
            String[] parts = name.split("-");
            String owner = parts.length > 0 ? parts[0] : name;
            boolean stale = false;
            if (owner.equals(packageName)) {
                boolean sameUpdate = parts.length == kept.length && parts[1].equals(kept[1]);
                boolean sameKind = parts.length == kept.length && parts[3].equals(kept[3]);
                stale = !sameUpdate || sameKind;
            } else if (checkAll && !isInstalled(context, owner)) {
                stale = true;
            }
            if (stale) {
                file.delete();
            }
        }
    }

    private static boolean isInstalled(Context context, String packageName) {
        try {
            context.getPackageManager().getPackageInfo(packageName, 0);
            return true;
        } catch (PackageManager.NameNotFoundException e) {
            return false;
        } catch (RuntimeException e) {
            return true;
        }
    }

    private static Drawable loadBanner(PackageManager pm, ComponentName component, String packageName) {
        try {
            Drawable banner = pm.getActivityBanner(component);
            return banner != null ? banner : pm.getApplicationBanner(packageName);
        } catch (PackageManager.NameNotFoundException | RuntimeException e) {
            return null;
        }
    }

    private static Drawable loadIcon(PackageManager pm, ComponentName component, String packageName) {
        // Prefer high density (DENSITY_XXHIGH / 480dpi) so dock icons (128x128) get sharp 144px+ assets
        // even on low-density TVs (like 720p tvdpi/213dpi). Limited to two attempts before system fallback.
        try {
            ActivityInfo info = pm.getActivityInfo(component, 0);
            int iconRes = info.getIconResource();
            if (iconRes != 0) {
                Resources res = pm.getResourcesForActivity(component);
                Drawable d = res.getDrawableForDensity(iconRes, DisplayMetrics.DENSITY_XXHIGH, null);
                if (d != null) {
                    return d;
                }
            }
        } catch (Exception ignored) {
            // Limited attempt: fall through to next attempt
        }
        try {
            ApplicationInfo appInfo = pm.getApplicationInfo(packageName, 0);
            int iconRes = appInfo.icon;
            if (iconRes != 0) {
                Resources res = pm.getResourcesForApplication(packageName);
                Drawable d = res.getDrawableForDensity(iconRes, DisplayMetrics.DENSITY_XXHIGH, null);
                if (d != null) {
                    return d;
                }
            }
        } catch (Exception ignored) {
            // Limited attempt: fall through to system fallback
        }
        try {
            return pm.getActivityIcon(component);
        } catch (PackageManager.NameNotFoundException | RuntimeException e) {
            try {
                return pm.getApplicationIcon(packageName);
            } catch (PackageManager.NameNotFoundException | RuntimeException again) {
                return null;
            }
        }
    }

    /** The banner stretched over the whole bitmap; lume's shader rounds the corners at the final size. */
    private static void drawBanner(Bitmap bitmap, Drawable drawable) {
        drawable.setBounds(0, 0, bitmap.getWidth(), bitmap.getHeight());
        drawable.draw(new Canvas(bitmap));
    }

    /** The icon in a square (`ICON_RATIO` of the height) centered in the bitmap, keeping its proportions. */
    private static void drawIcon(Bitmap bitmap, Drawable drawable) {
        int box = Math.round(bitmap.getHeight() * ICON_RATIO);
        int boxLeft = (bitmap.getWidth() - box) / 2;
        int boxTop = (bitmap.getHeight() - box) / 2;
        int w = drawable.getIntrinsicWidth();
        int h = drawable.getIntrinsicHeight();
        int left = boxLeft;
        int top = boxTop;
        int right = boxLeft + box;
        int bottom = boxTop + box;
        if (w > 0 && h > 0 && w != h) {
            if (w > h) {
                top = boxTop + (box - box * h / w) / 2;
                bottom = boxTop + box - (box - box * h / w) / 2;
            } else {
                left = boxLeft + (box - box * w / h) / 2;
                right = boxLeft + box - (box - box * w / h) / 2;
            }
        }
        drawable.setBounds(left, top, right, bottom);
        drawable.draw(new Canvas(bitmap));
    }

    /** Labels travel in a tab-separated, line-based format. */
    private static String clean(CharSequence text) {
        return text == null ? "" : text.toString().replace('\t', ' ').replace('\n', ' ').replace('\r', ' ');
    }

    /** The app's icon scaled to {@code ICON_SIZE} squared, as hex ARGB words. */
    private static String icon(Context context, String packageName) {
        try {
            Drawable drawable = context.getPackageManager().getApplicationIcon(packageName);
            Bitmap bitmap = Bitmap.createBitmap(ICON_SIZE, ICON_SIZE, Bitmap.Config.ARGB_8888);
            Canvas canvas = new Canvas(bitmap);
            drawable.setBounds(0, 0, ICON_SIZE, ICON_SIZE);
            drawable.draw(canvas);
            int[] pixels = new int[ICON_SIZE * ICON_SIZE];
            bitmap.getPixels(pixels, 0, ICON_SIZE, 0, 0, ICON_SIZE, ICON_SIZE);
            bitmap.recycle();
            StringBuilder out = new StringBuilder(pixels.length * 8);
            for (int pixel : pixels) {
                String hex = Integer.toHexString(pixel);
                for (int i = hex.length(); i < 8; i++) {
                    out.append('0');
                }
                out.append(hex);
            }
            return out.toString();
        } catch (PackageManager.NameNotFoundException e) {
            return "";
        }
    }

    /**
     * Packages that changed since the previous call, one per line, or "-" the first time (there
     * is no baseline yet, so the caller scans everything).
     */
    private static synchronized String changed(Context context) {
        PackageManager pm = context.getPackageManager();
        if (changeSequence < 0) {
            // First call of this process: the sequence of the last run is still good, as long as the
            // device did not reboot (the numbers start over then). Reading it spares a full scan,
            // which opens the resources of every installed app to read the labels.
            changeSequence = readSavedSequence(context);
            if (changeSequence < 0) {
                ChangedPackages baseline = pm.getChangedPackages(0);
                changeSequence = baseline == null ? 0 : baseline.getSequenceNumber();
                saveSequence(context, changeSequence);
                return "-";
            }
        }
        ChangedPackages changes = pm.getChangedPackages(changeSequence);
        if (changes == null) {
            return "";
        }
        changeSequence = changes.getSequenceNumber();
        saveSequence(context, changeSequence);
        return String.join("\n", changes.getPackageNames());
    }

    private static int bootCount(Context context) {
        return Settings.Global.getInt(context.getContentResolver(), Settings.Global.BOOT_COUNT, -1);
    }

    private static File sequenceFile(Context context) {
        return new File(context.getFilesDir(), SEQUENCE_FILE);
    }

    /** The saved sequence, or -1 when there is none, it is unreadable or it is from another boot. */
    private static int readSavedSequence(Context context) {
        int boot = bootCount(context);
        File file = sequenceFile(context);
        if (boot < 0 || !file.isFile() || file.length() > 64) {
            return -1;
        }
        try (FileInputStream in = new FileInputStream(file)) {
            byte[] bytes = new byte[(int) file.length()];
            if (in.read(bytes) != bytes.length) {
                return -1;
            }
            String[] parts = new String(bytes, StandardCharsets.UTF_8).trim().split("\t");
            if (parts.length != 2 || Integer.parseInt(parts[0]) != boot) {
                return -1;
            }
            return Math.max(-1, Integer.parseInt(parts[1]));
        } catch (IOException | RuntimeException e) {
            return -1;
        }
    }

    private static void saveSequence(Context context, int sequence) {
        int boot = bootCount(context);
        if (boot < 0) {
            return;
        }
        File file = sequenceFile(context);
        File temporary = new File(file.getPath() + ".tmp");
        try (FileOutputStream out = new FileOutputStream(temporary)) {
            out.write((boot + "\t" + sequence).getBytes(StandardCharsets.UTF_8));
        } catch (IOException e) {
            temporary.delete();
            return;
        }
        if (!temporary.renameTo(file)) {
            temporary.delete();
        }
    }

    /** Argument: package, a line break, and the activity class. Answers "ok" or "missing". */
    private static String start(Context context, String argument) {
        String[] parts = argument.split("\n", 2);
        if (parts.length != 2) {
            return "missing";
        }
        Intent intent = new Intent(Intent.ACTION_MAIN);
        intent.setComponent(new ComponentName(parts[0], parts[1]));
        intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_RESET_TASK_IF_NEEDED);
        try {
            context.startActivity(intent);
            return "ok";
        } catch (ActivityNotFoundException | SecurityException e) {
            return "missing";
        }
    }

    private static String openSettings(Context context) {
        Intent intent = new Intent(Settings.ACTION_SETTINGS).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
        return startOrMissing(context, intent);
    }

    /** The app's page in the Android settings. */
    private static String openAppInfo(Context context, String packageName) {
        Intent intent = new Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
                Uri.fromParts("package", packageName, null))
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
        return startOrMissing(context, intent);
    }

    /**
     * The system uninstaller for an app. It asks the user; the result arrives later as a package
     * change. Since Android 9 it needs REQUEST_DELETE_PACKAGES, or it does nothing.
     */
    private static String uninstall(Context context, String packageName) {
        Intent intent = new Intent(Intent.ACTION_DELETE, Uri.fromParts("package", packageName, null))
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
        return startOrMissing(context, intent);
    }

    private static String startOrMissing(Context context, Intent intent) {
        try {
            context.startActivity(intent);
            return "ok";
        } catch (ActivityNotFoundException | SecurityException e) {
            return "missing";
        }
    }

    private static final String ACTION_VIEW_INPUTS = "com.android.tv.action.VIEW_INPUTS";

    /** "1" when the system has a TV inputs panel, "0" otherwise. */
    private static String inputsAvailable(Context context) {
        Intent intent = new Intent(ACTION_VIEW_INPUTS);
        return intent.resolveActivity(context.getPackageManager()) != null ? "1" : "0";
    }

    private static String openInputs(Context context) {
        Intent intent = new Intent(ACTION_VIEW_INPUTS).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
        try {
            context.startActivity(intent);
            return "ok";
        } catch (ActivityNotFoundException | SecurityException e) {
            return "missing";
        }
    }

    /** TV model profile detected to prioritize the most pertinent validation sequence. */
    private enum DeviceProfile {
        TCL,
        AIWA,
        GENERIC
    }

    private static DeviceProfile cachedProfile = null;
    private static final Map<String, Intent> resolvedSettingCache = new ConcurrentHashMap<>();

    private static DeviceProfile getDeviceProfile() {
        if (cachedProfile != null) {
            return cachedProfile;
        }
        String info = (Build.MANUFACTURER + " " + Build.BRAND + " " + Build.MODEL + " " + Build.PRODUCT)
                .toLowerCase(Locale.ROOT);
        if (info.contains("tcl") || info.contains("beyondtv")) {
            cachedProfile = DeviceProfile.TCL;
        } else if (info.contains("aiwa") || info.contains("ktc") || info.contains("stanford") || info.contains("mediatek")) {
            cachedProfile = DeviceProfile.AIWA;
        } else {
            cachedProfile = DeviceProfile.GENERIC;
        }
        return cachedProfile;
    }

    /** An activity of the TV settings app, by name. */
    private static Intent tvSettings(String activity) {
        return new Intent().setClassName("com.android.tv.settings", "com.android.tv.settings." + activity);
    }

    /**
     * Whether `intent` leads to a real screen: something resolves it, and that is not the stub some
     * firmwares register to say "no app can do this" nor the chooser that asks which app to use.
     */
    private static boolean leadsToScreen(Context context, Intent intent) {
        ResolveInfo found = context.getPackageManager().resolveActivity(intent, 0);
        if (found == null || found.activityInfo == null) {
            return false;
        }
        if (found.activityInfo.name != null && found.activityInfo.name.endsWith("EmptyStubActivity")) {
            return false;
        }
        String owner = found.activityInfo.packageName;
        return !"android".equals(owner) && !"com.google.android.tv.frameworkpackagestubs".equals(owner);
    }

    /**
     * Constructs the intent cascade prioritized for the specific TV model profile.
     */
    private static Intent[] getIntentsForSetting(DeviceProfile profile, String setting) {
        switch (setting) {
            case "sleep":
                if (profile == DeviceProfile.AIWA) {
                    return new Intent[] {
                        tvSettings("partnercustomizer.timer.TimerActivity"),
                        tvSettings("device.display.daydream.DaydreamActivity"),
                        new Intent("android.settings.DREAM_SETTINGS"),
                        new Intent(Settings.ACTION_SETTINGS)
                    };
                } else if (profile == DeviceProfile.TCL) {
                    return new Intent[] {
                        tvSettings("device.display.daydream.DaydreamActivity"),
                        new Intent("android.settings.DREAM_SETTINGS"),
                        new Intent(Settings.ACTION_SETTINGS)
                    };
                } else {
                    return new Intent[] {
                        tvSettings("device.display.daydream.DaydreamActivity"),
                        tvSettings("partnercustomizer.timer.TimerActivity"),
                        new Intent("android.settings.DREAM_SETTINGS"),
                        new Intent(Settings.ACTION_SETTINGS)
                    };
                }

            case "picture":
                if (profile == DeviceProfile.AIWA) {
                    return new Intent[] {
                        new Intent("android.settings.PQAQ.SETTINGS"),
                        tvSettings("device.displaysound.DisplaySoundActivity"),
                        new Intent(Settings.ACTION_DISPLAY_SETTINGS),
                        new Intent(Settings.ACTION_SETTINGS)
                    };
                } else {
                    return new Intent[] {
                        tvSettings("device.displaysound.DisplaySoundActivity"),
                        new Intent(Settings.ACTION_DISPLAY_SETTINGS),
                        new Intent("android.settings.PQAQ.SETTINGS"),
                        new Intent(Settings.ACTION_SETTINGS)
                    };
                }

            case "sound":
                if (profile == DeviceProfile.TCL) {
                    // TCL TvSettings declares SoundActivity in manifest but lacks class in DEX.
                    // Fall back cleanly to DisplaySound or General Settings to avoid ROM crash logs.
                    return new Intent[] {
                        tvSettings("device.displaysound.DisplaySoundActivity"),
                        new Intent(Settings.ACTION_SETTINGS)
                    };
                } else {
                    return new Intent[] {
                        new Intent(Settings.ACTION_SOUND_SETTINGS),
                        tvSettings("device.displaysound.DisplaySoundActivity"),
                        new Intent(Settings.ACTION_SETTINGS)
                    };
                }

            case "wifi":
                return new Intent[] {
                    new Intent(Settings.ACTION_WIFI_SETTINGS),
                    tvSettings("connectivity.NetworkActivity"),
                    new Intent(Settings.ACTION_WIRELESS_SETTINGS),
                    new Intent(Settings.ACTION_SETTINGS)
                };

            case "bluetooth":
                return new Intent[] {
                    new Intent("com.google.android.intent.action.CONNECT_INPUT"),
                    tvSettings("accessories.AddAccessoryActivity"),
                    new Intent(Settings.ACTION_BLUETOOTH_SETTINGS),
                    new Intent(Settings.ACTION_SETTINGS)
                };

            case "apps":
                // The standard action is answered by a chooser on some TVs, so the screen comes first.
                return new Intent[] {
                    tvSettings("device.apps.AppsActivity"),
                    tvSettings("device.apps.AllAppsActivity"),
                    new Intent(Settings.ACTION_APPLICATION_SETTINGS),
                    new Intent(Settings.ACTION_MANAGE_APPLICATIONS_SETTINGS),
                    new Intent(Settings.ACTION_SETTINGS)
                };

            case "storage":
                return new Intent[] {
                    new Intent("com.android.tv.settings.STORAGE"),
                    tvSettings("device.StorageSummaryActivity"),
                    new Intent(Settings.ACTION_INTERNAL_STORAGE_SETTINGS),
                    new Intent(Settings.ACTION_MEMORY_CARD_SETTINGS),
                    new Intent(Settings.ACTION_SETTINGS)
                };

            case "language":
                return new Intent[] {
                    new Intent(Settings.ACTION_LOCALE_SETTINGS),
                    tvSettings("system.LanguageActivity"),
                    new Intent(Settings.ACTION_SETTINGS)
                };

            case "datetime":
                // The standard action is answered by a chooser on some TVs, so the screen comes first.
                return new Intent[] {
                    tvSettings("system.DateTimeActivity"),
                    new Intent(Settings.ACTION_DATE_SETTINGS),
                    new Intent(Settings.ACTION_SETTINGS)
                };

            case "keyboard":
                return new Intent[] {
                    new Intent(Settings.ACTION_INPUT_METHOD_SETTINGS),
                    tvSettings("inputmethod.KeyboardActivity"),
                    new Intent(Settings.ACTION_SETTINGS)
                };

            case "captions":
                return new Intent[] {
                    new Intent(Settings.ACTION_CAPTIONING_SETTINGS),
                    tvSettings("system.CaptionSetupActivity"),
                    new Intent(Settings.ACTION_SETTINGS)
                };

            case "location":
                return new Intent[] {
                    new Intent(Settings.ACTION_LOCATION_SOURCE_SETTINGS),
                    tvSettings("system.LocationActivity"),
                    new Intent(Settings.ACTION_SETTINGS)
                };

            case "security":
                return new Intent[] {
                    new Intent(Settings.ACTION_SECURITY_SETTINGS),
                    tvSettings("system.SecurityActivity"),
                    new Intent(Settings.ACTION_SETTINGS)
                };

            case "developer":
                return new Intent[] {
                    new Intent(Settings.ACTION_APPLICATION_DEVELOPMENT_SETTINGS),
                    tvSettings("system.development.DevelopmentActivity"),
                    new Intent(Settings.ACTION_SETTINGS)
                };

            case "home_app":
                return new Intent[] {
                    new Intent(Settings.ACTION_HOME_SETTINGS).setPackage("com.google.android.permissioncontroller"),
                    new Intent(Settings.ACTION_HOME_SETTINGS),
                    new Intent(Settings.ACTION_SETTINGS)
                };

            case "about":
                return new Intent[] {
                    new Intent(Settings.ACTION_DEVICE_INFO_SETTINGS),
                    tvSettings("about.AboutActivity"),
                    new Intent(Settings.ACTION_SETTINGS)
                };

            case "android":
            default:
                return new Intent[] {
                    new Intent(Settings.ACTION_SETTINGS)
                };
        }
    }

    /**
     * Opens the settings screen of a shortcut. Uses TV model detection to prioritize the most pertinent
     * sequence of validation (AIWA, TCL, and Android TV default), with caching of working intents.
     */
    private static String openSetting(Context context, String setting) {
        if (setting == null) {
            return "missing";
        }

        // Fast path: reuse working intent resolved previously
        Intent cached = resolvedSettingCache.get(setting);
        if (cached != null) {
            try {
                context.startActivity(cached);
                return "ok";
            } catch (Exception e) {
                resolvedSettingCache.remove(setting);
            }
        }

        DeviceProfile profile = getDeviceProfile();
        Intent[] intents = getIntentsForSetting(profile, setting);

        for (Intent intent : intents) {
            intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
            if (leadsToScreen(context, intent)) {
                try {
                    context.startActivity(intent);
                    resolvedSettingCache.put(setting, new Intent(intent));
                    return "ok";
                } catch (Exception ignored) {
                }
            }
        }
        try {
            Intent fallback = new Intent(Settings.ACTION_SETTINGS).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
            context.startActivity(fallback);
            return "ok";
        } catch (Exception e) {
            return "missing";
        }
    }

    private static boolean statsLogged = false;

    /**
     * Memory, data disk and CPU, for the settings panel: five tab-separated fields, namely total and
     * available memory in KB, total and free disk in bytes, and the CPU reading (see {@link #cpu}).
     */
    private static String stats(Context context) {
        ActivityManager.MemoryInfo memory = new ActivityManager.MemoryInfo();
        ((ActivityManager) context.getSystemService(Context.ACTIVITY_SERVICE)).getMemoryInfo(memory);
        StatFs disk = new StatFs(Environment.getDataDirectory().getPath());
        String cpu = cpu();
        if (!statsLogged) {
            statsLogged = true;
            Log.i("lume", "stats: cpu source " + cpu.split(",")[0]);
        }
        return (memory.totalMem / 1024) + "\t" + (memory.availMem / 1024) + "\t"
                + disk.getTotalBytes() + "\t" + disk.getAvailableBytes() + "\t" + cpu;
    }

    /**
     * The CPU reading: "stat,busy,total" in jiffies from /proc/stat, or, where the system blocks it
     * (Android 8 and later usually do), "freq,current,max" in kHz summed over the cores, or "-".
     */
    private static String cpu() {
        try (BufferedReader reader = new BufferedReader(new FileReader("/proc/stat"))) {
            String[] fields = reader.readLine().trim().split("\\s+");
            long total = 0;
            for (int i = 1; i < fields.length; i++) {
                total += Long.parseLong(fields[i]);
            }
            long idle = Long.parseLong(fields[4]) + Long.parseLong(fields[5]);
            return "stat," + (total - idle) + "," + total;
        } catch (IOException | RuntimeException e) {
            // Fall through to the frequencies.
        }
        long current = 0;
        long max = 0;
        for (int core = 0; core < 16; core++) {
            String base = "/sys/devices/system/cpu/cpu" + core + "/cpufreq/";
            long cur = readNumber(base + "scaling_cur_freq");
            long top = readNumber(base + "cpuinfo_max_freq");
            if (cur >= 0 && top > 0) {
                current += cur;
                max += top;
            }
        }
        return max > 0 ? "freq," + current + "," + max : "-";
    }

    private static long readNumber(String path) {
        try (BufferedReader reader = new BufferedReader(new FileReader(path))) {
            return Long.parseLong(reader.readLine().trim());
        } catch (IOException | RuntimeException e) {
            return -1;
        }
    }

    /** "1" or "0" for the 24-hour setting, a tab, and the UTC offset of the time zone in seconds. */
    private static String clock(Context context) {
        String is24 = DateFormat.is24HourFormat(context) ? "1" : "0";
        int offsetSeconds = TimeZone.getDefault().getOffset(System.currentTimeMillis()) / 1000;
        return is24 + "\t" + offsetSeconds;
    }

    /** The interface width and height in pixels, the refresh rate in Hz and the density in dpi, tab-separated. */
    private static String display(Context context) {
        DisplayMetrics metrics = context.getResources().getDisplayMetrics();
        WindowManager windows = (WindowManager) context.getSystemService(Context.WINDOW_SERVICE);
        float hertz = windows.getDefaultDisplay().getRefreshRate();
        return metrics.widthPixels + "\t" + metrics.heightPixels + "\t" + hertz + "\t" + metrics.densityDpi;
    }
}
