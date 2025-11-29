package dev.rgb.testagent.mixin;

import dev.rgb.testagent.RgbTestAgentMod;
import dev.rgb.testagent.TestAgentServer;
import dev.rgb.testagent.protocol.TestEvent;
import net.minecraft.client.network.ClientLoginNetworkHandler;
import net.minecraft.network.packet.s2c.login.LoginSuccessS2CPacket;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ClientLoginNetworkHandler.class)
public class ClientLoginNetworkHandlerMixin {

    @Inject(method = "onSuccess", at = @At("HEAD"))
    private void onLoginSuccess(LoginSuccessS2CPacket packet, CallbackInfo ci) {
        TestAgentServer server = RgbTestAgentMod.INSTANCE.getServer();
        if (server != null) {
            server.sendEvent(new TestEvent.LoginSuccess(
                    "login_success",
                    packet.profile().id().toString(),
                    packet.profile().name()
            ));
        }
    }
}
