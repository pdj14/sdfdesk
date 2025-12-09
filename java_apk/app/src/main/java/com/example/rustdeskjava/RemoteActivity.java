package com.example.rustdeskjava;

import android.app.Activity;
import android.content.Context;
import android.content.Intent;
import android.graphics.Bitmap;
import android.graphics.Canvas;
import android.graphics.Rect;
import android.os.Bundle;
import android.util.Log;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.View;
import android.widget.ImageButton;
import android.widget.Toast;
import java.nio.ByteBuffer;

public class RemoteActivity extends Activity implements SurfaceHolder.Callback {
    private static final String TAG = "RemoteActivity";
    private static final String EXTRA_REMOTE_ID = "remote_id";
    private static final String EXTRA_PASSWORD = "password";
    private static final String EXTRA_FILE_TRANSFER = "file_transfer";
    
    private String remoteId;
    private String password;
    private boolean isFileTransfer;
    private SurfaceView surfaceView;
    private SurfaceHolder surfaceHolder;
    private boolean isRunning = false;
    private Thread renderThread;
    private ByteBuffer rgbaBuffer;
    private Bitmap bitmap;

    public static void start(Context context, String remoteId, String password, boolean isFileTransfer) {
        Log.i(TAG, "=== start() CALLED ===");
        Log.i(TAG, "Context: " + context);
        Log.i(TAG, "Remote ID: " + remoteId);
        Log.i(TAG, "Password length: " + (password != null ? password.length() : 0));
        Log.i(TAG, "Is File Transfer: " + isFileTransfer);
        
        try {
            Intent intent = new Intent(context, RemoteActivity.class);
            intent.putExtra(EXTRA_REMOTE_ID, remoteId);
            intent.putExtra(EXTRA_PASSWORD, password);
            intent.putExtra(EXTRA_FILE_TRANSFER, isFileTransfer);
            Log.d(TAG, "Intent created, starting activity...");
            context.startActivity(intent);
            Log.d(TAG, "startActivity() called successfully");
        } catch (Exception e) {
            Log.e(TAG, "Error in start()", e);
            throw e;
        }
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        Log.i(TAG, "=== onCreate() CALLED ===");
        
        remoteId = getIntent().getStringExtra(EXTRA_REMOTE_ID);
        password = getIntent().getStringExtra(EXTRA_PASSWORD);
        isFileTransfer = getIntent().getBooleanExtra(EXTRA_FILE_TRANSFER, false);
        
        Log.d(TAG, "Remote ID from intent: " + remoteId);
        Log.d(TAG, "Password length from intent: " + (password != null ? password.length() : 0));
        Log.d(TAG, "Is File Transfer from intent: " + isFileTransfer);
        
        if (remoteId == null || remoteId.isEmpty()) {
            Log.e(TAG, "Invalid remote ID, finishing activity");
            Toast.makeText(this, "Invalid remote ID", Toast.LENGTH_SHORT).show();
            finish();
            return;
        }

        // Create simple layout programmatically
        Log.d(TAG, "Creating SurfaceView");
        surfaceView = new SurfaceView(this);
        setContentView(surfaceView);
        
        surfaceHolder = surfaceView.getHolder();
        surfaceHolder.addCallback(this);
        Log.d(TAG, "SurfaceHolder callback added");
        
        // Start connection
        startConnection();
    }

    private void startConnection() {
        Log.i(TAG, "=== startConnection() CALLED ===");
        Log.d(TAG, "Remote ID: " + remoteId);
        Log.d(TAG, "Password: " + (password != null ? "[" + password.length() + " chars]" : "null"));
        
        try {
            Log.d(TAG, "Calling RustDeskCore.startSession()...");
            RustDeskCore.startSession(remoteId, password != null ? password : "");
            Log.d(TAG, "RustDeskCore.startSession() returned successfully");
            
            isRunning = true;
            Log.d(TAG, "Starting render thread...");
            startRenderThread();
            Log.d(TAG, "Render thread started");
        } catch (UnsatisfiedLinkError e) {
            Log.e(TAG, "JNI Error - Native method not found", e);
            Toast.makeText(this, "JNI Error: " + e.getMessage(), Toast.LENGTH_LONG).show();
            finish();
        } catch (Exception e) {
            Log.e(TAG, "Failed to start session", e);
            Toast.makeText(this, "Connection failed: " + e.getMessage(), Toast.LENGTH_SHORT).show();
            finish();
        }
    }

    private void startRenderThread() {
        Log.d(TAG, "Creating render thread");
        renderThread = new Thread(() -> {
            Log.d(TAG, "Render thread running");
            int frameCount = 0;
            while (isRunning) {
                try {
                    renderFrame();
                    frameCount++;
                    if (frameCount % 60 == 0) {
                        Log.d(TAG, "Rendered " + frameCount + " frames");
                    }
                    Thread.sleep(16); // ~60 FPS
                } catch (InterruptedException e) {
                    Log.d(TAG, "Render thread interrupted");
                    break;
                } catch (Exception e) {
                    Log.e(TAG, "Render error", e);
                }
            }
            Log.d(TAG, "Render thread stopped");
        });
        renderThread.start();
    }

    private int logCounter = 0;
    
    private void renderFrame() {
        try {
            int size = RustDeskCore.getRgbaSize(remoteId);
            int width = RustDeskCore.getWidth(remoteId);
            int height = RustDeskCore.getHeight(remoteId);
            
            // Log every 60 frames
            logCounter++;
            if (logCounter % 60 == 1) {
                Log.d(TAG, "Frame data - size: " + size + ", width: " + width + ", height: " + height);
            }
            
            if (size > 0 && width > 0 && height > 0) {
                if (rgbaBuffer == null || rgbaBuffer.capacity() != size) {
                    Log.d(TAG, "Creating new buffer: size=" + size + ", " + width + "x" + height);
                    rgbaBuffer = ByteBuffer.allocateDirect(size);
                }
                
                RustDeskCore.getRgba(remoteId, 0, rgbaBuffer);
                
                if (bitmap == null || bitmap.getWidth() != width || bitmap.getHeight() != height) {
                    Log.d(TAG, "Creating new bitmap: " + width + "x" + height);
                    bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888);
                }
                
                rgbaBuffer.rewind();
                bitmap.copyPixelsFromBuffer(rgbaBuffer);
                
                if (surfaceHolder != null && surfaceHolder.getSurface().isValid()) {
                    Canvas canvas = surfaceHolder.lockCanvas();
                    if (canvas != null) {
                        Rect src = new Rect(0, 0, width, height);
                        Rect dst = new Rect(0, 0, canvas.getWidth(), canvas.getHeight());
                        canvas.drawBitmap(bitmap, src, dst, null);
                        surfaceHolder.unlockCanvasAndPost(canvas);
                    }
                }
            }
        } catch (UnsatisfiedLinkError e) {
            Log.e(TAG, "JNI Error in renderFrame", e);
            isRunning = false;
        } catch (Exception e) {
            Log.e(TAG, "Error rendering frame", e);
        }
    }

    @Override
    public boolean onTouchEvent(MotionEvent event) {
        Log.v(TAG, "Touch event: " + event.getAction() + " at (" + event.getX() + ", " + event.getY() + ")");
        // TODO: Send touch events to remote
        return super.onTouchEvent(event);
    }

    @Override
    public void surfaceCreated(SurfaceHolder holder) {
        Log.d(TAG, "Surface created");
    }

    @Override
    public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
        Log.d(TAG, "Surface changed: " + width + "x" + height + ", format=" + format);
    }

    @Override
    public void surfaceDestroyed(SurfaceHolder holder) {
        Log.d(TAG, "Surface destroyed");
    }

    @Override
    protected void onDestroy() {
        Log.i(TAG, "=== onDestroy() CALLED ===");
        super.onDestroy();
        isRunning = false;
        if (renderThread != null) {
            try {
                Log.d(TAG, "Waiting for render thread to stop...");
                renderThread.join(1000);
                Log.d(TAG, "Render thread stopped");
            } catch (InterruptedException e) {
                Log.e(TAG, "Error stopping render thread", e);
            }
        }
        try {
            Log.d(TAG, "Calling RustDeskCore.stopSession()...");
            RustDeskCore.stopSession(remoteId);
            Log.d(TAG, "Session stopped");
        } catch (Exception e) {
            Log.e(TAG, "Error stopping session", e);
        }
    }

    @Override
    public void onBackPressed() {
        Log.d(TAG, "Back button pressed");
        super.onBackPressed();
        finish();
    }
}
