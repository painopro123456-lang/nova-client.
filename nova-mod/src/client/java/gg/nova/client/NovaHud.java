package gg.nova.client;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.DeltaTracker;
import net.minecraft.world.entity.EquipmentSlot;
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
            drawPanel(graphics, 8, 8, value, TEXT);
        }

        if (NovaModules.enabled("Keystrokes")) {
            drawKey(graphics, 33, 35, "W", client.options.keyUp.isDown());
            drawKey(graphics, 8, 60, "A", client.options.keyLeft.isDown());
            drawKey(graphics, 33, 60, "S", client.options.keyDown.isDown());
            drawKey(graphics, 58, 60, "D", client.options.keyRight.isDown());
        }

        if (NovaModules.enabled("Clock")) {
            drawPanel(graphics, 8, 88, LocalTime.now().format(CLOCK), MUTED);
        }

        if (NovaModules.enabled("Coordinates")) {
            var position = client.player.blockPosition();
            String value = "XYZ  " + position.getX() + " / " + position.getY() + " / " + position.getZ();
            drawPanel(graphics, 8, 116, value, TEXT);
        }

        if (NovaModules.enabled("Direction")) {
            float yaw = client.player.getYRot();
            String direction = directionFromYaw(yaw);
            drawPanel(graphics, 8, 144, "Facing  " + direction, MUTED);
        }

        if (NovaModules.enabled("Armor Status")) {
            int x = 8;
            int y = 172;
            EquipmentSlot[] slots = {EquipmentSlot.HEAD, EquipmentSlot.CHEST, EquipmentSlot.LEGS, EquipmentSlot.FEET};
            String[] labels = {"HELM", "CHEST", "LEGS", "BOOTS"};
            for (int i = 0; i < slots.length; i++) {
                var stack = client.player.getItemBySlot(slots[i]);
                String value = labels[i] + "  —";
                if (!stack.isEmpty() && stack.isDamageableItem()) {
                    int remaining = Math.max(0, stack.getMaxDamage() - stack.getDamageValue());
                    value = labels[i] + "  " + remaining;
                } else if (!stack.isEmpty()) {
                    value = labels[i] + "  ✓";
                }
                drawPanel(graphics, x, y, value, TEXT);
                y += 24;
            }
        }
    }

    private static String directionFromYaw(float yaw) {
        int index = Math.floorMod(Math.round(yaw / 45.0f), 8);
        return switch (index) {
            case 0 -> "S";
            case 1 -> "SW";
            case 2 -> "W";
            case 3 -> "NW";
            case 4 -> "N";
            case 5 -> "NE";
            case 6 -> "E";
            default -> "SE";
        };
    }

    private static void drawPanel(GuiGraphicsExtractor graphics, int x, int y, String value, int color) {
        int width = Math.max(78, Minecraft.getInstance().font.width(value) + 18);
        graphics.fill(x, y, x + width, y + 21, PANEL);
        graphics.fill(x, y, x + 3, y + 21, ACCENT);
        graphics.text(Minecraft.getInstance().font, value, x + 9, y + 7, color, true);
    }

    private static void drawKey(GuiGraphicsExtractor graphics, int x, int y, String label, boolean down) {
        graphics.fill(x, y, x + 22, y + 22, down ? 0xFF45376C : PANEL);
        graphics.fill(x, y, x + 22, y + 2, down ? ACCENT : 0xFF394153);
        graphics.text(Minecraft.getInstance().font, label, x + 7, y + 7, TEXT, true);
    }
}