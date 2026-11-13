package gg.nova.client;

import net.minecraft.client.Minecraft;
import net.minecraft.client.DeltaTracker;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.resources.Identifier;
import net.fabricmc.fabric.api.client.rendering.v1.hud.HudElementRegistry;
import net.fabricmc.fabric.api.client.rendering.v1.hud.VanillaHudElements;

import java.time.LocalTime;
import java.time.format.DateTimeFormatter;
import java.util.List;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.entity.EquipmentSlot;

public final class NovaHud {
    private static final DateTimeFormatter CLOCK=DateTimeFormatter.ofPattern("HH:mm:ss");
    private NovaHud() {}

    public static void register() {
        HudElementRegistry.attachElementBefore(
            VanillaHudElements.CHAT,
            Identifier.fromNamespaceAndPath("nova","hud_overlay"),
            NovaHud::render
        );
    }

    private static void line(GuiGraphicsExtractor graphics,Minecraft client,String module,String text,int x,int y) {
        if(NovaMenuScreen.hasBackground(module)) {
            graphics.fill(x-4,y-3,x+client.font.width(text)+4,y+client.font.lineHeight+3,0xA0111520);
        }
        graphics.text(client.font,text,x,y,0xFFF3F5FC,true);
    }

    private static void render(GuiGraphicsExtractor graphics,DeltaTracker delta) {
        Minecraft client=Minecraft.getInstance();
        if(client.player==null||client.level==null||client.gui.screen() instanceof NovaMenuScreen)return;
        int y=8;
        if(NovaMenuScreen.isEnabled("FPS")) {
            line(graphics,client,"FPS",client.getFps()+" FPS",8,y);y+=18;
        }
        if(NovaMenuScreen.isEnabled("Clock")) {
            String clock=LocalTime.now().format(CLOCK);
            int x=client.getWindow().getGuiScaledWidth()-client.font.width(clock)-12;
            line(graphics,client,"Clock",clock,x,8);
        }
        if(NovaMenuScreen.isEnabled("Keystrokes")) {
            String up=client.options.keyUp.isDown()?"W":"w";
            String left=client.options.keyLeft.isDown()?"A":"a";
            String down=client.options.keyDown.isDown()?"S":"s";
            String right=client.options.keyRight.isDown()?"D":"d";
            line(graphics,client,"Keystrokes","[ "+up+" ]",8,36);
            line(graphics,client,"Keystrokes","["+left+"] ["+down+"] ["+right+"]",8,54);
            line(graphics,client,"Keystrokes",client.options.keyJump.isDown()?"[ SPACE ]":"[ space ]",8,72);
        }
        if(NovaMenuScreen.isEnabled("Armor Status")) {
            int armorY=94;
            for(EquipmentSlot slot:List.of(EquipmentSlot.FEET,EquipmentSlot.LEGS,EquipmentSlot.CHEST,EquipmentSlot.HEAD)) {
                ItemStack stack=client.player.getItemBySlot(slot);
                if(stack.isEmpty())continue;
                int remaining=Math.max(0,stack.getMaxDamage()-stack.getDamageValue());
                line(graphics,client,"Armor Status",stack.getHoverName().getString()+"  "+remaining,8,armorY);
                armorY+=18;
            }
        }
    }
}
