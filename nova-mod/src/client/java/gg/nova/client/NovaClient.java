package gg.nova.client;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keymapping.v1.KeyMappingHelper;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.gui.screens.Screen;
import com.mojang.blaze3d.platform.InputConstants;
import net.minecraft.resources.Identifier;
public final class NovaClient implements ClientModInitializer {
    private KeyMapping openMenuKey;
    @Override public void onInitializeClient() {
        KeyMapping.Category category = KeyMapping.Category.register(Identifier.fromNamespaceAndPath("nova", "main"));
        openMenuKey = KeyMappingHelper.registerKeyMapping(new KeyMapping("key.nova.open_menu", InputConstants.Type.KEYSYM, InputConstants.KEY_RIGHT_SHIFT, category));
        ClientTickEvents.END_CLIENT_TICK.register(client -> {
            while (openMenuKey.consumeClick()) {
                Screen current = client.gui.screen();
                if (current instanceof NovaMenuScreen) client.gui.setScreen(null);
                else if (client.player != null) client.gui.setScreen(new NovaMenuScreen());
            }
        });
    }
}