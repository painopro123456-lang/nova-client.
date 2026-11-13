package gg.nova.client;

import net.fabricmc.loader.api.FabricLoader;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Properties;

public final class NovaModules {
    private static final Properties SETTINGS = new Properties();
    private static final Path FILE = FabricLoader.getInstance().getConfigDir().resolve("nova-modules.properties");

    static {
        try {
            if (Files.isRegularFile(FILE)) {
                try (InputStream input = Files.newInputStream(FILE)) {
                    SETTINGS.load(input);
                }
            }
        } catch (IOException ignored) {
            // Defaults remain available if the settings file cannot be read.
        }
    }

    private NovaModules() {}

    public static boolean enabled(String name) {
        String fallback = switch (name) {
            case "FPS", "Keystrokes", "Coordinates" -> "true";
            default -> "false";
        };
        return Boolean.parseBoolean(SETTINGS.getProperty(name, fallback));
    }

    public static void setEnabled(String name, boolean enabled) {
        SETTINGS.setProperty(name, Boolean.toString(enabled));
        try {
            Files.createDirectories(FILE.getParent());
            try (OutputStream output = Files.newOutputStream(FILE)) {
                SETTINGS.store(output, "NOVA Client module settings");
            }
        } catch (IOException ignored) {
            // Keep the current session usable if persistence fails.
        }
    }
}