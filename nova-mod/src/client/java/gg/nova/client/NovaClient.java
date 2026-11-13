package gg.nova.client;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.minecraft.client.KeyMapping;
import org.lwjgl.glfw.GLFW;

public final class NovaClient implements ClientModInitializer {
    private KeyMapping openMenuKey;
    @Override public void onInitializeClient() {
        openMenuKey=KeyBindingHelper.registerKeyBinding(new KeyMapping("key.nova.open_menu",GLFW.GLFW_KEY_RIGHT_SHIFT,KeyMapping.Category.MISC));
        NovaMenuScreen.initialize();
        NovaHud.register();
        ClientTickEvents.END_CLIENT_TICK.register(client->{
            while(openMenuKey.consumeClick()){
                if(client.screen instanceof NovaMenuScreen)client.setScreen(null);
                else if(client.player!=null)client.setScreen(new NovaMenuScreen());
            }
        });
    }
}
