package gg.nova.client;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keymapping.v1.KeyMappingHelper;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.gui.screens.Screen;
import com.mojang.blaze3d.platform.InputConstants;
import org.lwjgl.glfw.GLFW;
import net.minecraft.resources.Identifier;
import net.fabricmc.fabric.api.client.rendering.v1.hud.HudElementRegistry;
import net.fabricmc.fabric.api.client.screen.v1.ScreenEvents;
import net.fabricmc.fabric.api.client.screen.v1.ScreenKeyboardEvents;
public final class NovaClient implements ClientModInitializer {
    private KeyMapping openMenuKey;
    @Override public void onInitializeClient() {
        HudElementRegistry.addLast(Identifier.fromNamespaceAndPath("nova", "overlay"), NovaHud::render);
        KeyMapping.Category category = KeyMapping.Category.register(Identifier.fromNamespaceAndPath("nova", "main"));
        openMenuKey = KeyMappingHelper.registerKeyMapping(new KeyMapping("key.nova.open_menu", InputConstants.Type.KEYSYM, GLFW.GLFW_KEY_RIGHT_SHIFT, category));
        ClientTickEvents.END_CLIENT_TICK.register(client -> {
            while (openMenuKey.consumeClick()) {
                if (client.gui.screen() == null && client.player != null) {
                    client.gui.setScreen(new NovaMenuScreen());
                }
            }
        });
        ScreenEvents.BEFORE_INIT.register((client, screen, scaledWidth, scaledHeight) -> {
            if (screen instanceof NovaMenuScreen) {
                ScreenKeyboardEvents.beforeKeyPress(screen).register((current, keyEvent) -> {
                    if (openMenuKey.matches(keyEvent)) {
                        while (openMenuKey.consumeClick()) { }
                        client.gui.setScreen(null);
                    }
                });
            }
        });
    }
}