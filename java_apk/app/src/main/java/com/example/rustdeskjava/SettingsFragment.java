package com.example.rustdeskjava;

import android.os.Bundle;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;
import android.widget.Toast;
import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.fragment.app.Fragment;
import com.google.android.material.button.MaterialButton;
import com.google.android.material.textfield.TextInputEditText;

public class SettingsFragment extends Fragment {
    private TextInputEditText idServerInput;
    private TextInputEditText relayServerInput;
    private TextInputEditText apiServerInput;
    private TextInputEditText keyInput;
    private MaterialButton btnSaveSettings;
    private TextView appVersion;

    @Nullable
    @Override
    public View onCreateView(@NonNull LayoutInflater inflater, @Nullable ViewGroup container, @Nullable Bundle savedInstanceState) {
        View view = inflater.inflate(R.layout.fragment_settings, container, false);
        
        idServerInput = view.findViewById(R.id.id_server_input);
        relayServerInput = view.findViewById(R.id.relay_server_input);
        apiServerInput = view.findViewById(R.id.api_server_input);
        keyInput = view.findViewById(R.id.key_input);
        btnSaveSettings = view.findViewById(R.id.btn_save_settings);
        appVersion = view.findViewById(R.id.app_version);
        
        loadSettings();
        setupListeners();
        
        return view;
    }

    private void loadSettings() {
        // Load settings from RustDeskCore
        try {
            // TODO: Implement getLocalOption in RustDeskCore
            // idServerInput.setText(RustDeskCore.getLocalOption("custom-rendezvous-server"));
            // relayServerInput.setText(RustDeskCore.getLocalOption("relay-server"));
            // apiServerInput.setText(RustDeskCore.getLocalOption("api-server"));
            // keyInput.setText(RustDeskCore.getLocalOption("key"));
            
            appVersion.setText("RustDesk Java Client v1.0.0");
        } catch (Exception e) {
            appVersion.setText("Error loading settings: " + e.getMessage());
        }
    }

    private void setupListeners() {
        btnSaveSettings.setOnClickListener(v -> saveSettings());
    }

    private void saveSettings() {
        try {
            String idServer = idServerInput.getText() != null ? idServerInput.getText().toString() : "";
            String relayServer = relayServerInput.getText() != null ? relayServerInput.getText().toString() : "";
            String apiServer = apiServerInput.getText() != null ? apiServerInput.getText().toString() : "";
            String key = keyInput.getText() != null ? keyInput.getText().toString() : "";
            
            // TODO: Implement setLocalOption in RustDeskCore
            // RustDeskCore.setLocalOption("custom-rendezvous-server", idServer);
            // RustDeskCore.setLocalOption("relay-server", relayServer);
            // RustDeskCore.setLocalOption("api-server", apiServer);
            // RustDeskCore.setLocalOption("key", key);
            
            if (getContext() != null) {
                Toast.makeText(getContext(), "Settings saved", Toast.LENGTH_SHORT).show();
            }
        } catch (Exception e) {
            if (getContext() != null) {
                Toast.makeText(getContext(), "Error saving settings: " + e.getMessage(), Toast.LENGTH_SHORT).show();
            }
        }
    }
}
