package gg.nova.client;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.DeltaTracker;
import java.time.LocalTime;
import java.time.format.DateTimeFormatter;

public final class NovaHud {
    private static final int PANEL = 0xB90D101A;
    private static final int ACCENT = 0xFF9B7CFF;
    private static final int TEXT = 0xFFF3F5FA;
    private static final int MUTED = 0xFF929CB2;
    private static final DateTimeFormatter CLOCK = DateTimeFormatter.ofPattern("HH:mm:ss");

    private NovaHud() {}

    public static void render(GuiGraphicsExtractor graphics, DeltaTracker deltaTracker) {
        Minecraft client = Minecraft.getInstance();
        if (client.player == null) return;

        if (NovaModules.enabled("FPS")) {
            String value = client.getFps() + " FPS";
            graphics.fill(8, 8, 86, 29, PANEL);
            graphics.fill(8, 8, 11, 29, ACCENT);
            graphics.text(client.font, value, 17, 14, TEXT, true);
        }

        if (NovaModules.enabled("Keystrokes")) {
            drawKey(graphics, 33, 35, "W", client.options.keyUp.isDown());
            drawKey(graphics, 8, 60, "A", client.options.keyLeft.isDown());
            drawKey(graphics, 33, 60, "S", client.options.keyDown.isDown());
            drawKey(graphics, 58, 60, "D", client.options.keyRight.isDown());
        }

        if (NovaModules.enabled("Clock")) {
            String value = LocalTime.now().format(CLOCK);
            graphics.fill(8, 88, 86, 109, PANEL);
            graphics.fill(8, 88, 11, 109, ACCENT);
            graphics.text(client.font, value, 17, 94, MUTED, true);
        }
    }

    private static void drawKey(GuiGraphicsExtractor graphics, int x, int y, String label, boolean down) {
        graphics.fill(x, y, x + 22, y + 22, down ? 0xFF45376C : PANEL);
        graphics.fill(x, y, x + 22, y + 2, down ? ACCENT : 0xFF394153);
        graphics.text(Minecraft.getInstance().font, label, x + 7, y + 7, TEXT, true);
    }
}