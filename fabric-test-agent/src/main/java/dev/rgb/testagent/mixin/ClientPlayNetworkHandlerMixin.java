package dev.rgb.testagent.mixin;

import dev.rgb.testagent.RgbTestAgentMod;
import dev.rgb.testagent.TestAgentServer;
import dev.rgb.testagent.protocol.TestEvent;
import net.minecraft.client.network.ClientPlayNetworkHandler;
import net.minecraft.network.packet.s2c.play.GameJoinS2CPacket;
import net.minecraft.network.packet.s2c.play.PlayerPositionLookS2CPacket;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ClientPlayNetworkHandler.class)
public class ClientPlayNetworkHandlerMixin {

    @Inject(method = "onGameJoin", at = @At("TAIL"))
    private void onGameJoin(GameJoinS2CPacket packet, CallbackInfo ci) {
        TestAgentServer server = RgbTestAgentMod.INSTANCE.getServer();
        if (server != null) {
            server.sendEvent(new TestEvent.PlayState(
                    "play_state",
                    packet.playerEntityId()
            ));
        }
    }

    @Inject(method = "onPlayerPositionLook", at = @At("TAIL"))
    private void onPositionLook(PlayerPositionLookS2CPacket packet, CallbackInfo ci) {
        TestAgentServer server = RgbTestAgentMod.INSTANCE.getServer();
        if (server != null) {
            server.sendEvent(new TestEvent.PositionSync(
                    "position_sync",
                    packet.change().position().x,
                    packet.change().position().y,
                    packet.change().position().z,
                    packet.change().yaw(),
                    packet.change().pitch()
            ));
        }
    }
}
