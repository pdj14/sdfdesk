package com.example.rustdeskjava;

import android.app.AlertDialog;
import android.os.Bundle;
import android.text.InputType;
import android.util.Log;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.EditText;
import android.widget.TextView;
import android.widget.Toast;
import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.fragment.app.Fragment;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;
import com.google.android.material.button.MaterialButton;
import com.google.android.material.textfield.TextInputEditText;

public class ConnectionFragment extends Fragment {
    private static final String TAG = "ConnectionFragment";
    
    private TextInputEditText remoteIdInput;
    private MaterialButton btnConnect;
    private MaterialButton btnFileTransfer;
    private TextView statusText;
    private RecyclerView recentPeersList;

    @Nullable
    @Override
    public View onCreateView(@NonNull LayoutInflater inflater, @Nullable ViewGroup container, @Nullable Bundle savedInstanceState) {
        Log.d(TAG, "onCreateView called");
        View view = inflater.inflate(R.layout.fragment_connection, container, false);
        
        remoteIdInput = view.findViewById(R.id.remote_id_input);
        btnConnect = view.findViewById(R.id.btn_connect);
        btnFileTransfer = view.findViewById(R.id.btn_file_transfer);
        statusText = view.findViewById(R.id.status_text);
        recentPeersList = view.findViewById(R.id.recent_peers_list);
        
        Log.d(TAG, "Views found - remoteIdInput: " + (remoteIdInput != null) + 
                    ", btnConnect: " + (btnConnect != null) + 
                    ", statusText: " + (statusText != null));
        
        setupListeners();
        setupRecyclerView();
        
        return view;
    }

    private void setupListeners() {
        Log.d(TAG, "Setting up listeners");
        
        btnConnect.setOnClickListener(v -> {
            Log.d(TAG, "Connect button clicked");
            String remoteId = remoteIdInput.getText() != null ? remoteIdInput.getText().toString().trim() : "";
            Log.d(TAG, "Remote ID entered: '" + remoteId + "'");
            
            if (remoteId.isEmpty()) {
                Log.w(TAG, "Remote ID is empty");
                statusText.setText("Please enter a remote ID");
                return;
            }
            showPasswordDialog(remoteId, false);
        });

        btnFileTransfer.setOnClickListener(v -> {
            Log.d(TAG, "File transfer button clicked");
            String remoteId = remoteIdInput.getText() != null ? remoteIdInput.getText().toString().trim() : "";
            Log.d(TAG, "Remote ID for file transfer: '" + remoteId + "'");
            
            if (remoteId.isEmpty()) {
                Log.w(TAG, "Remote ID is empty for file transfer");
                statusText.setText("Please enter a remote ID");
                return;
            }
            showPasswordDialog(remoteId, true);
        });
    }

    private void showPasswordDialog(String remoteId, boolean isFileTransfer) {
        Log.d(TAG, "Showing password dialog for: " + remoteId);
        
        if (getContext() == null) {
            Log.e(TAG, "Context is null, cannot show dialog");
            return;
        }

        EditText passwordInput = new EditText(getContext());
        passwordInput.setHint("Enter password");
        passwordInput.setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_VARIATION_PASSWORD);
        passwordInput.setPadding(50, 30, 50, 30);

        new AlertDialog.Builder(getContext())
            .setTitle("Connect to " + remoteId)
            .setMessage("Enter the password for the remote device:")
            .setView(passwordInput)
            .setPositiveButton("Connect", (dialog, which) -> {
                String password = passwordInput.getText().toString();
                Log.d(TAG, "Password entered, length: " + password.length());
                connect(remoteId, password, isFileTransfer);
            })
            .setNegativeButton("Cancel", (dialog, which) -> {
                Log.d(TAG, "Password dialog cancelled");
                dialog.dismiss();
            })
            .show();
    }

    private void setupRecyclerView() {
        Log.d(TAG, "Setting up RecyclerView");
        recentPeersList.setLayoutManager(new LinearLayoutManager(getContext()));
        // TODO: Add adapter for recent peers
    }

    private void connect(String remoteId, String password, boolean isFileTransfer) {
        Log.i(TAG, "=== CONNECT CALLED ===");
        Log.i(TAG, "Remote ID: " + remoteId);
        Log.i(TAG, "Password length: " + password.length());
        Log.i(TAG, "Is File Transfer: " + isFileTransfer);
        
        statusText.setText("Connecting to " + remoteId + "...");
        
        // Start RemoteActivity
        if (getActivity() != null) {
            Log.d(TAG, "Activity is not null, starting RemoteActivity");
            try {
                RemoteActivity.start(getActivity(), remoteId, password, isFileTransfer);
                Log.d(TAG, "RemoteActivity.start() called successfully");
            } catch (Exception e) {
                Log.e(TAG, "Error starting RemoteActivity", e);
                statusText.setText("Error: " + e.getMessage());
            }
        } else {
            Log.e(TAG, "Activity is null, cannot start RemoteActivity");
            statusText.setText("Error: Activity is null");
        }
    }
}
