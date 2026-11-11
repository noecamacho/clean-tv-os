# Rust (liblume.so) reaches these through JNI by name, so R8 must neither rename nor remove them.
-keep class app.lume.tv.LumeActivity {
    public java.lang.String bridge(java.lang.String, java.lang.String);
    public byte[] iconPixels(java.lang.String, java.lang.String, int, int);
    public byte[] dockIcon(java.lang.String, java.lang.String, int);
    private static native void nativeHomePressed();
}
-keep class app.lume.tv.LumeBridge { *; }
