package gg.nova.client;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Properties;

public final class NovaMenuScreen extends Screen {
    private static final int PANEL=0xF20D101A, CARD=0xFF171D2B, ACCENT=0xFF9B7CFF, MUTED=0xFF929CB2;
    private static final Map<String,Boolean> modules=new LinkedHashMap<>();
    private static final Map<String,String> categories=new LinkedHashMap<>();
    private static final Map<String,Boolean> backgrounds=new LinkedHashMap<>();
    private String category="All";
    private String settingsModule=null;

    public NovaMenuScreen() {
        super(Component.literal("NOVA"));
        initialize();
    }

    public static void initialize() {
        if(!modules.isEmpty())return;
        addModule("Keystrokes",true,"HUD");
        addModule("FPS",true,"HUD");
        addModule("Armor Status",false,"HUD");
        addModule("Clock",false,"HUD");
        loadStates();
    }

    private static void addModule(String name, boolean enabled, String group) {
        modules.put(name,enabled);
        categories.put(name,group);
        backgrounds.put(name,true);
    }

    private static Path statePath() {
        return Minecraft.getInstance().gameDirectory.toPath().resolve("config").resolve("nova-modules.properties");
    }

    private static void loadStates() {
        Path path=statePath();
        if(!Files.isRegularFile(path))return;
        Properties values=new Properties();
        try(InputStream input=Files.newInputStream(path)) {
            values.load(input);
            for(String name:modules.keySet()) {
                String stored=values.getProperty(name);
                if(stored!=null)modules.put(name,Boolean.parseBoolean(stored));
                String background=values.getProperty(name+".background");
                if(background!=null)backgrounds.put(name,Boolean.parseBoolean(background));
            }
        } catch(IOException ignored) { }
    }

    public static boolean isEnabled(String name) { return modules.getOrDefault(name,false); }
    public static boolean hasBackground(String name) { return backgrounds.getOrDefault(name,true); }

    private static void saveStates() {
        Properties values=new Properties();
        modules.forEach((name,enabled)->values.setProperty(name,Boolean.toString(enabled)));
        backgrounds.forEach((name,enabled)->values.setProperty(name+".background",Boolean.toString(enabled)));
        try {
            Files.createDirectories(statePath().getParent());
            try(OutputStream output=Files.newOutputStream(statePath())) {
                values.store(output,"NOVA module preferences");
            }
        } catch(IOException ignored) { }
    }

    @Override
    public void extractRenderState(GuiGraphicsExtractor g,int mx,int my,float delta) {
        g.fill(0,0,width,height,0xB9000000);
        int pw=Math.min(760,width-40), ph=Math.min(480,height-40), l=(width-pw)/2, t=(height-ph)/2;
        g.fill(l,t,l+pw,t+ph,PANEL);
        g.fill(l,t,l+5,t+ph,ACCENT);
        g.text(font,"NOVA",l+26,t+22,0xFFFFFFFF,false);
        g.text(font,"CLIENT MENU",l+91,t+24,MUTED,false);
        g.text(font,"RIGHT SHIFT  -  CLOSE",l+pw-158,t+25,MUTED,false);
        g.fill(l+20,t+48,l+pw-20,t+49,0xFF303849);

        String[] cats={"All","HUD","Visual","Player","Mods"};
        for(int i=0;i<cats.length;i++) {
            int y=t+68+i*35;
            boolean selected=category.equals(cats[i]);
            if(selected)g.fill(l+20,y-5,l+152,y+22,0xFF29233E);
            g.text(font,cats[i],l+30,y+3,selected?ACCENT:0xFFD7DCE8,false);
        }

        int cx=l+178,cy=t+70,cw=(pw-220)/2;
        g.text(font,category.equals("All")?"Modules":category,cx,cy,0xFFFFFFFF,false);
        g.text(font,"Saved module preferences",cx,cy+17,MUTED,false);
        int visible=0;
        for(var entry:modules.entrySet()) {
            if(!category.equals("All")&&!category.equals(categories.get(entry.getKey())))continue;
            int col=visible%2,row=visible/2,x=cx+col*(cw+12),y=cy+48+row*72;
            g.fill(x,y,x+cw,y+62,CARD);
            g.text(font,entry.getKey(),x+12,y+13,0xFFFFFFFF,false);
            g.text(font,categories.get(entry.getKey())+" module",x+12,y+34,MUTED,false);
            g.text(font,"⚙",x+cw-19,y+13,0xFFD7DCE8,false);
            g.fill(x+cw-56,y+20,x+cw-28,y+34,entry.getValue()?ACCENT:0xFF414A5D);
            g.fill(x+cw-(entry.getValue()?40:54),y+22,x+cw-(entry.getValue()?30:44),y+32,0xFFFFFFFF);
            visible++;
        }
        if(visible==0)g.text(font,"No modules in this category yet.",cx,cy+58,MUTED,false);

        if(settingsModule!=null) {
            int sy=t+ph-76;
            g.fill(cx,sy,l+pw-24,sy+50,0xFF242B3B);
            g.text(font,"SETTINGS  ·  "+settingsModule,cx+12,sy+8,0xFFFFFFFF,false);
            g.text(font,"Background: "+(hasBackground(settingsModule)?"ON":"OFF")+"  ·  CLICK TO CHANGE",cx+12,sy+27,MUTED,false);
        }
        g.text(font,"NOVA IN-GAME  -  EARLY DEVELOPMENT",l+26,t+ph-24,MUTED,false);
        super.extractRenderState(g,mx,my,delta);
    }

    @Override
    public boolean mouseClicked(double mx,double my,int button) {
        if(button!=0)return super.mouseClicked(mx,my,button);
        int pw=Math.min(760,width-40),ph=Math.min(480,height-40),l=(width-pw)/2,t=(height-ph)/2;
        String[] cats={"All","HUD","Visual","Player","Mods"};
        for(int i=0;i<cats.length;i++) {
            int y=t+68+i*35;
            if(mx>=l+20&&mx<=l+152&&my>=y-5&&my<=y+22) {
                category=cats[i];settingsModule=null;return true;
            }
        }
        int cx=l+178,cy=t+70,cw=(pw-220)/2,visible=0;
        for(var entry:modules.entrySet()) {
            if(!category.equals("All")&&!category.equals(categories.get(entry.getKey())))continue;
            int col=visible%2,row=visible/2,x=cx+col*(cw+12),y=cy+48+row*72;
            if(mx>=x&&mx<=x+cw&&my>=y&&my<=y+62) {
                if(mx>=x+cw-26)settingsModule=entry.getKey();
                else {modules.put(entry.getKey(),!entry.getValue());saveStates();settingsModule=null;}
                return true;
            }
            visible++;
        }
        if(settingsModule!=null) {
            int sy=t+ph-76;
            if(mx>=cx&&mx<=l+pw-24&&my>=sy&&my<=sy+50) {
                backgrounds.put(settingsModule,!hasBackground(settingsModule));
                saveStates();
                return true;
            }
            settingsModule=null;return true;
        }
        return super.mouseClicked(mx,my,button);
    }

    @Override
    public boolean isPauseScreen() { return false; }
}
