package gg.nova.client;

import com.mojang.blaze3d.platform.InputConstants;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keymapping.v1.KeyMappingHelper;
import net.fabricmc.fabric.api.client.screen.v1.ScreenEvents;
import net.fabricmc.fabric.api.client.screen.v1.ScreenKeyboardEvents;
import net.minecraft.client.KeyMapping;
import net.minecraft.resources.Identifier;

public final class NovaClient implements ClientModInitializer {
    private KeyMapping openMenuKey;
    private boolean handledInScreen;
    private static final KeyMapping.Category CATEGORY=KeyMapping.Category.register(Identifier.fromNamespaceAndPath("nova","main"));

    @Override public void onInitializeClient() {
        openMenuKey=KeyMappingHelper.registerKeyMapping(new KeyMapping(
            "key.nova.open_menu",
            InputConstants.Type.KEYSYM,
            InputConstants.KEY_RIGHT_SHIFT,
            CATEGORY
        ));
        NovaMenuScreen.initialize();
        NovaHud.register();

        ClientTickEvents.END_CLIENT_TICK.register(client->{
            while(openMenuKey.consumeClick()){
                if(!handledInScreen&&client.gui.screen()==null&&client.player!=null)client.gui.setScreen(new NovaMenuScreen());
            }
            handledInScreen=false;
        });

        ScreenEvents.AFTER_INIT.register((client,screen,scaledWidth,scaledHeight)->{
            ScreenKeyboardEvents.afterKeyPress(screen).register((current,keyEvent)->{
                if(!openMenuKey.matches(keyEvent)||client.player==null)return;
                handledInScreen=true;
                if(current instanceof NovaMenuScreen)client.gui.setScreen(null);
                else client.gui.setScreen(new NovaMenuScreen());
            });
        });
    }
}
