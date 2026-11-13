package gg.nova.client;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;
import java.util.LinkedHashMap;
import java.util.Map;
public final class NovaMenuScreen extends Screen {
    private static final int PANEL=0xF20D101A, CARD=0xFF171D2B, ACCENT=0xFF9B7CFF, MUTED=0xFF929CB2;
    private final Map<String,Boolean> modules=new LinkedHashMap<>();
    private static final Map<String,String> moduleCategories=Map.of(
        "Keystrokes","HUD","FPS","HUD","CPS","HUD","Armor Status","HUD",
        "Potion Effects","HUD","Toggle Sprint","Player","Crosshair","Visual","Clock","HUD"
    );
    private String category="All";
    public NovaMenuScreen() {
        super(Component.literal("NOVA"));
        modules.put("Keystrokes",true); modules.put("FPS",true); modules.put("CPS",false);
        modules.put("Armor Status",false); modules.put("Potion Effects",false); modules.put("Toggle Sprint",false);
        modules.put("Crosshair",false); modules.put("Clock",false);
    }
    @Override public void render(GuiGraphics g,int mx,int my,float delta) {
        g.fill(0,0,width,height,0xB9000000);
        int pw=Math.min(760,width-40), ph=Math.min(480,height-40), l=(width-pw)/2, t=(height-ph)/2;
        g.fill(l,t,l+pw,t+ph,PANEL); g.fill(l,t,l+5,t+ph,ACCENT);
        g.drawString(font,"NOVA",l+26,t+22,0xFFFFFFFF,false);
        g.drawString(font,"CLIENT MENU",l+91,t+24,MUTED,false);
        g.drawString(font,"RIGHT SHIFT  -  CLOSE",l+pw-158,t+25,MUTED,false);
        g.fill(l+20,t+48,l+pw-20,t+49,0xFF303849);
        String[] cats={"All","HUD","Visual","Player","Mods"};
        for(int i=0;i<cats.length;i++){int y=t+68+i*35; boolean a=category.equals(cats[i]); if(a)g.fill(l+20,y-5,l+152,y+22,0xFF29233E); g.drawString(font,cats[i],l+30,y+3,a?ACCENT:0xFFD7DCE8,false);}
        int cx=l+178, cy=t+70, cw=(pw-220)/2;
        g.drawString(font,category.equals("All")?"Modules":category,cx,cy,0xFFFFFFFF,false);
        g.drawString(font,"Clean tools. Your game, your layout.",cx,cy+17,MUTED,false);
        int i=0;
        for(var e:modules.entrySet()){
            if(!category.equals("All")&&!category.equals(moduleCategories.get(e.getKey()))) continue;
            int col=i%2,row=i/2,x=cx+col*(cw+12),y=cy+48+row*72;
            g.fill(x,y,x+cw,y+62,CARD);
            g.drawString(font,e.getKey(),x+12,y+13,0xFFFFFFFF,false);
            g.drawString(font,"Preview toggle · feature not active yet",x+12,y+34,MUTED,false);
            g.fill(x+cw-42,y+20,x+cw-14,y+34,e.getValue()?ACCENT:0xFF414A5D);
            g.fill(x+cw-(e.getValue()?26:40),y+22,x+cw-(e.getValue()?16:30),y+32,0xFFFFFFFF);
            i++;
        }
        if(i==0) g.drawString(font,"No modules in this category yet.",cx,cy+58,MUTED,false);
        g.drawString(font,"NOVA IN-GAME  -  EARLY DEVELOPMENT",l+26,t+ph-24,MUTED,false);
        super.render(g,mx,my,delta);
    }
    @Override public boolean mouseClicked(double mx,double my,int button) {
        if(button!=0)return super.mouseClicked(mx,my,button);
        int pw=Math.min(760,width-40),ph=Math.min(480,height-40),l=(width-pw)/2,t=(height-ph)/2;
        String[] cats={"All","HUD","Visual","Player","Mods"};
        for(int i=0;i<cats.length;i++){int y=t+68+i*35;if(mx>=l+20&&mx<=l+152&&my>=y-5&&my<=y+22){category=cats[i];return true;}}
        int cx=l+178,cy=t+70,cw=(pw-220)/2,i=0;
        for(var e:modules.entrySet()){
            if(!category.equals("All")&&!category.equals(moduleCategories.get(e.getKey()))) continue;
            int col=i%2,row=i/2,x=cx+col*(cw+12),y=cy+48+row*72;
            if(mx>=x&&mx<=x+cw&&my>=y&&my<=y+62){modules.put(e.getKey(),!e.getValue());return true;}
            i++;
        }
        return super.mouseClicked(mx,my,button);
    }
    @Override public boolean isPauseScreen(){return false;}
}
