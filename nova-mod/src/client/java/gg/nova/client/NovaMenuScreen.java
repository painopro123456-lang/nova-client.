package gg.nova.client;

import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;
import java.util.Map;

public final class NovaMenuScreen extends Screen {
    private static final int PANEL=0xF20D101A, ACCENT=0xFF9B7CFF, MUTED=0xFF929CB2;

    private static final Map<String,String> CATEGORIES=Map.of(
        "Keystrokes","HUD", "FPS","HUD", "Clock","HUD",
        "Coordinates","HUD", "Direction","HUD"
    );
    private String category="All";

    public NovaMenuScreen() {
        super(Component.literal("NOVA Client"));
    }

    @Override protected void init() { rebuildWidgets(); }

    @Override protected void rebuildWidgets() {
        clearWidgets();
        int pw=Math.min(760,width-40), ph=Math.min(480,height-40), l=(width-pw)/2, t=(height-ph)/2;
        String[] cats={"All","HUD","Visual","Player","Mods"};
        for(int i=0;i<cats.length;i++) {
            String value=cats[i];
            addRenderableWidget(Button.builder(Component.literal(value),b->{category=value;rebuildWidgets();})
                .bounds(l+20,t+65+i*34,130,26).build());
        }
        int cx=l+178, cy=t+112, cw=Math.max(100,(pw-220)/2), index=0;
        for(String name:CATEGORIES.keySet()) {
            if(!category.equals("All")&&!category.equals(CATEGORIES.get(name))) continue;
            int col=index%2,row=index/2,x=cx+col*(cw+12),y=cy+row*48;
            boolean enabled=NovaModules.enabled(name);
            addRenderableWidget(Button.builder(Component.literal(name+"  ·  "+(enabled?"ON":"OFF")),b->{
                boolean next=!NovaModules.enabled(name); NovaModules.setEnabled(name,next);
                b.setMessage(Component.literal(name+"  ·  "+(next?"ON":"OFF")));
            }).bounds(x,y,cw,34).build());
            index++;
        }
    }

    @Override public void extractRenderState(GuiGraphicsExtractor g,int mx,int my,float delta) {
        int pw=Math.min(760,width-40),ph=Math.min(480,height-40),l=(width-pw)/2,t=(height-ph)/2;
        g.fill(0,0,width,height,0xB9000000); g.fill(l,t,l+pw,t+ph,PANEL); g.fill(l,t,l+5,t+ph,ACCENT);
        super.extractRenderState(g,mx,my,delta);
        g.text(font,"NOVA",l+26,t+22,0xFFFFFFFF,false);
        g.text(font,"CLIENT MENU",l+91,t+24,MUTED,false);
        g.text(font,"RIGHT SHIFT  -  CLOSE",l+pw-158,t+25,MUTED,false);
        g.fill(l+20,t+48,l+pw-20,t+49,0xFF303849);
        g.text(font,category.equals("All")?"Modules":category,l+178,t+91,0xFFFFFFFF,false);
        g.text(font,"5 live HUD modules · changes save automatically",l+178,t+ph-48,MUTED,false);
        g.text(font,"NOVA IN-GAME  -  EARLY DEVELOPMENT",l+26,t+ph-24,MUTED,false);
    }

    @Override public boolean isPauseScreen(){return false;}
}