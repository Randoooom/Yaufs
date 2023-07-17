/*
 *    Copyright  2023.  Fritz Ochsmann
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use mcproto_rs::{types::*, uuid::*, *};

// based on https://github.com/Twister915/mcproto-rs/blob/master/src/v1_16_3.rs
define_protocol!(762, Packet762, RawPacket762, RawPacket762Body, Packet762Kind => {
    Handshake, 0x00, Handshaking, ServerBound => HandshakeSpec {
        version: VarInt,
        server_address: String,
        server_port: u16,
        next_state: HandshakeNextState
    },
    // status
    StatusRequest, 0x00, Status, ServerBound => StatusRequestSpec {},
    StatusPing, 0x01, Status, ServerBound => StatusPingSpec {
        payload: i64
    },
    StatusResponse, 0x00, Status, ClientBound => StatusResponseSpec {
        response: super::status::StatusSpec
    },
    StatusPong, 0x01, Status, ClientBound => StatusPongSpec {
        payload: i64
    },

    // login
    LoginDisconnect, 0x00, Login, ClientBound => LoginDisconnectSpec {
        message: Chat
    },
    LoginEncryptionRequest, 0x01, Login, ClientBound => LoginEncryptionRequestSpec {
        server_id: String,
        public_key: CountedArray<u8, VarInt>,
        verify_token: CountedArray<u8, VarInt>
    },
    LoginSuccess, 0x02, Login, ClientBound => LoginSuccessSpec {
        uuid: UUID4,
        username: String,
        properties: CountedArray<LoginSuccessPropertiesSpec, VarInt>
    },
    LoginSetCompression, 0x03, Login, ClientBound => LoginSetCompressionSpec {
        threshold: VarInt
    },
    LoginPluginRequest, 0x04, Login, ClientBound => LoginPluginRequestSpec {
        message_id: VarInt,
        channel: String,
        data: RemainingBytes
    },
    LoginStart, 0x00, Login, ServerBound => LoginStartSpec {
        name: String,
        has_uuid: bool,
        uuid: UUID4
    },
    LoginEncryptionResponse, 0x01, Login, ServerBound => LoginEncryptionResponseSpec {
        shared_secret: CountedArray<u8, VarInt>,
        verify_token: CountedArray<u8, VarInt>
    },
    LoginPluginResponse, 0x02, Login, ServerBound => LoginPluginResponseSpec {
        message_id: VarInt,
        successful: bool,
        data: RemainingBytes
    },


    // play
    // client bound1
    PlayBundleDelimiter, 0x00, Play, ClientBound => PlayBundleDelimiterSpec {
        data: RemainingBytes
    },
    PlaySpawnEntity, 0x01, Play, ClientBound => PlaySpawnLivingEntitySpec {
        data: RemainingBytes
    },
    PlaySpawnExperienceOrb, 0x02, Play, ClientBound => PlaySpawnExperienceOrbSpec {
        data: RemainingBytes
    },
    PlaySpawnPlayer, 0x03, Play, ClientBound => PlaySpawnPlayerSpec {
        data: RemainingBytes
    },
    PlayEntityAnimation, 0x04, Play, ClientBound => PlayEntityAnimationSpec {
        data: RemainingBytes
    },
    PlayAwardStatistics, 0x05, Play, ClientBound => PlayAwardStatisticsSpec {
        data: RemainingBytes
    },
    PlayAcknowledgeBlockChange, 0x06, Play, ClientBound => PlayAcknowledgeBlockChangeSpec {
        data: RemainingBytes
    },
    PlaySetBlockDestroyStage, 0x07, Play, ClientBound => PlaySetBlockDestroyStageSpec {
        data: RemainingBytes
    },
    PlayBlockEntityData, 0x08, Play, ClientBound => PlayBlockEntityDataSpec {
        data: RemainingBytes
    },
    PlayBlockAction, 0x09, Play, ClientBound => PlayBlockActionSpec {
        data: RemainingBytes
    },
    PlayBlockUpdate, 0x0A, Play, ClientBound => PlayBlockUpdateSpec {
        data: RemainingBytes
    },
    PlayBossBar, 0x0B, Play, ClientBound => PlayBossBarSpec {
        data: RemainingBytes
    },
    PlayChangeDifficulty, 0x0C, Play, ClientBound => PlayChangeDifficultySpec {
        data: RemainingBytes
    },
    PlayChunkBiomes, 0x0D, Play, ClientBound => PlayChunkBiomesSpec {
        data: RemainingBytes
    },
    PlayClearTitles, 0x0E, Play, ClientBound => PlayClearTitlesSpec {
        data: RemainingBytes
    },
    PlayTabComplete, 0x0F, Play, ClientBound => PlayTabCompleteSpec {
        data: RemainingBytes
    },
    PlayDeclareCommands, 0x10, Play, ClientBound => PlayDeclareCommandsSpec {
        data: RemainingBytes
    },
    PlayCloseContainer, 0x11, Play, ClientBound => PlayCloseContainerSpec {
        data: RemainingBytes
    },
    PlaySetContainerContent, 0x12, Play, ClientBound => PlaySetContainerContentSpec {
        data: RemainingBytes
    },
    PlaySetContainerProperty, 0x13, Play, ClientBound => PlaySetContainerPropertySpec {
        data: RemainingBytes
    },
    PlaySetContainerSlot, 0x14, Play, ClientBound => PlaySetContainerSlotSpec {
        data: RemainingBytes
    },
    PlaySetCooldown, 0x15, Play, ClientBound => PlaySetCooldownSpec {
        data: RemainingBytes
    },
    PlayChatSuggestion, 0x16, Play, ClientBound => PlayChatSuggestionSpec {
        data: RemainingBytes
    },
    PlayServerPluginMessage, 0x17, Play, ClientBound => PlayServerPluginMessageSpec {
        data: RemainingBytes
    },
    PlayDamageEvent, 0x18, Play, ClientBound => PlayDamageEventSpec {
        data: RemainingBytes
    },
    PlayDeleteMessage, 0x19, Play, ClientBound => PlayDeleteMessageSpec {
        data: RemainingBytes
    },
    PlayDisconnect, 0x1A, Play, ClientBound => PlayDisconnectSpec {
        data: RemainingBytes
    },
    PlayDisguisedChatMessage, 0x1B, Play, ClientBound => PlayDisguisedChatMessageSpec {
        data: RemainingBytes
    },
    PlayEntityEvent, 0x1C, Play, ClientBound => PlayEntityEventSpec {
        data: RemainingBytes
    },
    PlayExplosion, 0x1D, Play, ClientBound => PlayExplosionSpec {
        data: RemainingBytes
    },
    PlayUnloadChunk, 0x1E, Play, ClientBound => PlayUnloadChunkSpec {
        data: RemainingBytes
    },
    PlayGameEvent, 0x1F, Play, ClientBound => PlayGameEventSpec {
        data: RemainingBytes
    },
    PlayOpenHorseScreen, 0x20, Play, ClientBound => PlayOpenHorseScreenSpec {
        data: RemainingBytes
    },
    PlayHurtAnimation, 0x21, Play, ClientBound => PlayHurtAnimationSpec {
        data: RemainingBytes
    },
    PlayInitializeWorldBorder, 0x22, Play, ClientBound => PlayInitializeWorldBorderSpec {
        data: RemainingBytes
    },
    PlayServerKeepAlive, 0x23, Play, ClientBound => PlayServerKeepAliveSpec {
        data: RemainingBytes
    },
    PlayChunkDataAndUpdateLight, 0x24, Play, ClientBound => PlayChunkDataAndUpdateLightSpec {
        data: RemainingBytes
    },
    PlayWorldEvent, 0x25, Play, ClientBound => PlayWorldEventSpec {
        data: RemainingBytes
    },
    PlayParticle, 0x26, Play, ClientBound => PlayParticleSpec {
        data: RemainingBytes
    },
    PlayUpdateLight, 0x27, Play, ClientBound => PlayUpdateLightSpec {
        data: RemainingBytes
    },
    PlayLogin, 0x28, Play, ClientBound => PlayLoginSpec {
        data: RemainingBytes
    },
    PlayMapData, 0x29, Play, ClientBound => PlayMapDataSpec {
        data: RemainingBytes
    },
    PlayMerchantOffers, 0x2A, Play, ClientBound => PlayMerchantOffersSpec {
        data: RemainingBytes
    },
    PlayUpdateEntityPosition, 0x2B, Play, ClientBound => PlayUpdateEntityPositionSpec {
        data: RemainingBytes
    },
    PlayUpdateEntityPositionAndRotation, 0x2C, Play, ClientBound => PlayUpdateEntityPositionAndRotationSpec {
        data: RemainingBytes
    },
    PlayUpdateEntityRotation, 0x2D, Play, ClientBound => PlayUpdateEntityRotationSpec {
        data: RemainingBytes
    },
    PlayMoveVehicle, 0x2E, Play, ClientBound => PlayMoveVehicleSpec {
        data: RemainingBytes
    },
    PlayOpenBook, 0x2F, Play, ClientBound => PlayOpenBookSpec {
        data: RemainingBytes
    },
    PlayerOpenScreen, 0x30, Play, ClientBound => PlayerOpenScreenSpec {
        data: RemainingBytes
    },
    PlayOpenSignEditor, 0x31, Play, ClientBound => PlayOpenSignEditorSpec {
        data: RemainingBytes
    },
    PlayPing, 0x32, Play, ClientBound => PlayPingSpec {
        data: RemainingBytes
    },
    PlayPlaceGhostRecipe, 0x33, Play, ClientBound => PlayPlaceGhostRecipeSpec {
        data: RemainingBytes
    },
    PlayPlayerAbilities, 0x34, Play, ClientBound => PlayPlayerAbilitiesSpec {
        data: RemainingBytes
    },
    PlayPlayerChatMessage, 0x35, Play, ClientBound => PlayPlayerChatMessageSpec {
        data: RemainingBytes
    },
    PlayEndCombat, 0x36, Play, ClientBound => PlayEndCombatSpec {
        data: RemainingBytes
    },
    PlayEnterCombat, 0x37, Play, ClientBound => PlayEnterCombatSpec {
        data: RemainingBytes
    },
    PlayCombatDeath, 0x38, Play, ClientBound => PlayCombatDeathSpec {
        data: RemainingBytes
    },
    PlayPlayerInfoRemove, 0x39, Play, ClientBound => PlayPlayerInfoRemoveSpec {
        data: RemainingBytes
    },
    PlayPlayerInfoUpdate, 0x3A, Play, ClientBound => PlayPlayerInfoUpdateSpec {
        data: RemainingBytes
    },
    PlayLookAt, 0x3B, Play, ClientBound => PlayLookAtSpec {
        data: RemainingBytes
    },
    PlaySynchronizePlayerPosition, 0x3C, Play, ClientBound => PlaySynchronizePlayerPositionSpec {
        data: RemainingBytes
    },
    PlayUpdateRecipeBook, 0x3D, Play, ClientBound => PlayUpdateRecipeBookSpec {
        data: RemainingBytes
    },
    PlayRemoveEntities, 0x3E, Play, ClientBound => PlayRemoveEntitiesSpec {
        data: RemainingBytes
    },
    PlayRemoveEntitiesEffect, 0x3F, Play, ClientBound => PlayRemoveEntities {
        data: RemainingBytes
    },
    PlayResourcePack, 0x40, Play, ClientBound => PlayResourcePackSpec {
        data: RemainingBytes
    },
    PlayRespawn, 0x41, Play, ClientBound => PlayRespawnSpec {
        data: RemainingBytes
    },
    PlaySetHeadRotation, 0x42, Play, ClientBound => PlaySetHeadRotationSpec {
        data: RemainingBytes
    },
    PlayUpdateSectionBlock, 0x43, Play, ClientBound => PlayUpdateSectionBlockSpec {
        data: RemainingBytes
    },
    PlaySelectAdvancementsTab, 0x44, Play, ClientBound => PlaySelectAdvancementsTabSpec {
        data: RemainingBytes
    },
    PlayServerData, 0x45, Play, ClientBound => PlayServerDataSpec {
        data: RemainingBytes
    },
    PlaySetActionBarText, 0x46, Play, ClientBound => PlaySetActionBarTextSpec {
        data: RemainingBytes
    },
    PlaySetBorderCenter, 0x47, Play, ClientBound => PlaySetBorderCenterSpec {
        data: RemainingBytes
    },
    PlaySetBorderLerpSize, 0x48, Play, ClientBound => PlaySetBorderLerpSizeSpec {
        data: RemainingBytes
    },
    PlaySetBorderSize, 0x49, Play, ClientBound => PlaySetBorderSizeSpec {
        data: RemainingBytes
    },
    PlaySetBorderWarningDelay, 0x4A, Play, ClientBound => PlaySetBorderWarningDelaySpec {
        data: RemainingBytes
    },
    PlaySetBorderWarningDistance, 0x4B, Play, ClientBound => PlaySetBorderWarningDistanceSpec {
        data: RemainingBytes
    },
    PlayTeams, 0x4C, Play, ClientBound => PlayTeamsSpec {
        data: RemainingBytes
    },
    PlaySetHeldItem, 0x4D, Play, ClientBound => PlaySetHeldItemSpec {
        data: RemainingBytes
    },
    PlayTimeUpdate, 0x4E, Play, ClientBound => PlayTimeUpdateSpec {
        data: RemainingBytes
    },
    PlayTitle, 0x4F, Play, ClientBound => PlayTitleSpec {
        data: RemainingBytes
    },
    PlaySetDefaultSpawnLocation, 0x50, Play, ClientBound => PlaySetDefaultSpawnLocationSpec {
        data: RemainingBytes
    },
    PlayDisplayObjective, 0x51, Play, ClientBound => PlayDisplayObjectiveSpec {
        data: RemainingBytes
    },
    PlaySetEntityMetadata, 0x52, Play, ClientBound => PlaySetEntityMetadataSpec {
        data: RemainingBytes
    },
    PlayLinkEntities, 0x53, Play, ClientBound => PlayLinkEntitiesSpec {
        data: RemainingBytes
    },
    PlaySetEntityVelocity, 0x54, Play, ClientBound => PlaySetEntityVelocitySpec {
        data: RemainingBytes
    },
    PlaySetEquipment, 0x55, Play, ClientBound => PlaySetEquipSpec {
        data: RemainingBytes
    },
    PlaySetExperience, 0x56, Play, ClientBound => PlaySetExperienceSpec {
        data: RemainingBytes
    },
    PlaySetHealth, 0x57, Play, ClientBound => PlaySetHealthSpec {
        data: RemainingBytes
    },
    PlayUpdateObjectives, 0x58, Play, ClientBound => PlayUpdateObjectivesSpec {
        data: RemainingBytes
    },
    PlaySetPassengers, 0x59, Play, ClientBound => PlaySetPassengersSpec {
        data: RemainingBytes
    },
    PlayUpdateTeams, 0x5A, Play, ClientBound => PlayUpdateTeamsSpec {
        data: RemainingBytes
    },
    PlayUpdateScore, 0x5B, Play, ClientBound => PlayUpdateScoreSpec {
        data: RemainingBytes
    },
    PlaySetSimulationDistance, 0x5C, Play, ClientBound => PlaySetSimulationDistanceSpec {
        data: RemainingBytes
    },
    PlaySetSubtitleText, 0x5D, Play, ClientBound => PlaySetSubtitleTextSpec {
        data: RemainingBytes
    },
    PlayUpdateTime, 0x5E, Play, ClientBound => PlayUpdateTimeSpec {
        data: RemainingBytes
    },
    PlaySetTitleText, 0x5F, Play, ClientBound => PlaySetTitleTextSpec {
        data: RemainingBytes
    },
    PlaySetTitleAnimationTimes, 0x60, Play, ClientBound => PlaySetTitleAnimationTimesSpec {
        data: RemainingBytes
    },
    PlayEntitySoundEffect, 0x61, Play, ClientBound => PlayEntitySoundEffectSpec {
        data: RemainingBytes
    },
    PlaySoundEffect, 0x62, Play, ClientBound => PlaySoundEffectSpec {
        data: RemainingBytes
    },
    PlayStopSound, 0x63, Play, ClientBound => PlayStopSoundSpec {
        data: RemainingBytes
    },
    PlaySystemChatMessage, 0x64, Play, ClientBound => PlaySystemChatMessageSpec {
        data: RemainingBytes
    },
    PlaySetTabListHeaderAndFooter, 0x65, Play, ClientBound => PlaySetTabListHeaderAndFooter {
        data: RemainingBytes
    },
    PlayTagQueryResponse, 0x66, Play, ClientBound => PlaySetTagQueryResponse {
        data: RemainingBytes
    },
    PlayPickupItem, 0x67, Play, ClientBound => PlayPickupItemSpec {
        data: RemainingBytes
    },
    PlayTeleportEntity, 0x68, Play, ClientBound => PlayTeleportEntitySpec {
        data: RemainingBytes
    },
    PlayUpdateAdvancements, 0x69, Play, ClientBound => PlayUpdateAdvancementsSpec {
        data: RemainingBytes
    },
    PlayUpdateAttributes, 0x6A, Play, ClientBound => PlayUpdateAttributesSpec {
        data: RemainingBytes
    },
    PlayFeatureFlags, 0x6B, Play, ClientBound => PlayFeatureFlagsSpec {
        data: RemainingBytes
    },
    PlayEntityEffect, 0x6C, Play, ClientBound => PlayEntityEffectSpec {
        data: RemainingBytes
    },
    PlayUpdateRecipes, 0x6D, Play, ClientBound => PlayUpdateRecipesSpec {
        data: RemainingBytes
    },
    PlayUpdateTags, 0x6E, Play, ClientBound => PlayUpdateTagsSpec {
        data: RemainingBytes
    },

    // play server bound
    PlayTeleportConfirm, 0x00, Play, ServerBound => PlayTeleportConfirmSpec {
        data: RemainingBytes
    },
    PlayQueryBlockNbt, 0x01, Play, ServerBound => PlayQueryBlockNbtSpec {
        data: RemainingBytes
    },
    PlayQueryEntityNbt, 0x0D, Play, ServerBound => PlayQueryEntityNbtSpec {
        data: RemainingBytes
    },
    PlaySetDifficulty, 0x02, Play, ServerBound => PlaySetDifficultySpec {
        data: RemainingBytes
    },
    PlayClientChatMessage, 0x03, Play, ServerBound => PlayClientChatMessageSpec {
        data: RemainingBytes
    },
    PlayClientStatus, 0x04, Play, ServerBound => PlayClientStatusSpec {
        data: RemainingBytes
    },
    PlayClientSettings, 0x05, Play, ServerBound => PlayClientSettingsSpec {
        data: RemainingBytes
    },
    PlayClientTabComplete, 0x06, Play, ServerBound => PlayClientTabCompleteSpec {
        data: RemainingBytes
    },
    PlayClientWindowConfirmation, 0x07, Play, ServerBound => PlayClientWindowConfirmationSpec {
        data: RemainingBytes
    },
    PlayClickWindowButton, 0x08, Play, ServerBound => PlayClickWindowButtonSpec {
        data: RemainingBytes
    },
    PlayClickWindow, 0x09, Play, ServerBound => PlayClickWindowSpec {
        data: RemainingBytes
    },
    PlayClientCloseWindow, 0x0A, Play, ServerBound => PlayClientCloseWindowSpec {
        data: RemainingBytes
    },
    PlayClientPluginMessage, 0x0B, Play, ServerBound => PlayClientPluginMessageSpec {
        data: RemainingBytes
    },
    PlayEditBook, 0x0C, Play, ServerBound => PlayEditBookSpec {
        data: RemainingBytes
    },
    PlayInteractEntity, 0x0E, Play, ServerBound => PlayInteractEntitySpec {
        data: RemainingBytes
    },
    PlayGenerateStructure, 0x0F, Play, ServerBound => PlayGenerateStructureSpec {
        data: RemainingBytes
    },
    PlayClientKeepAlive, 0x10, Play, ServerBound => PlayClientKeepAliveSpec {
        data: RemainingBytes
    },
    PlayLockDifficulty, 0x11, Play, ServerBound => PlayLockDifficultySpec {
        data: RemainingBytes
    },
    PlayPlayerPosition, 0x12, Play, ServerBound => PlayPlayerPositionSpec {
        data: RemainingBytes
    },
    PlayClientPlayerPositionAndRotation, 0x13, Play, ServerBound => PlayClientPlayerPositionAndRotationSpec {
        data: RemainingBytes
    },
    PlayPlayerRotation, 0x14, Play, ServerBound => PlayPlayerRotationSpec {
        data: RemainingBytes
    },
    PlayPlayerMovement, 0x15, Play, ServerBound => PlayPlayerMovementSpec {
        data: RemainingBytes
    },
    PlayClientVehicleMove, 0x16, Play, ServerBound => PlayClientVehicleMoveSpec {
        data: RemainingBytes
    },
    PlaySteerBoat, 0x17, Play, ServerBound => PlaySteerBoatSpec {
        data: RemainingBytes
    },
    PlayPickItem, 0x18, Play, ServerBound => PlayPickItemSpec {
        data: RemainingBytes
    },
    PlayCraftRecipeRequest, 0x19, Play, ServerBound => PlayCraftRecipeRequestSpec {
        data: RemainingBytes
    },
    PlayClientPlayerAbilities, 0x1A, Play, ServerBound => PlayClientPlayerAbilitiesSpec {
        data: RemainingBytes
    },
    PlayPlayerDigging, 0x1B, Play, ServerBound => PlayPlayerDiggingSpec {
        data: RemainingBytes
    },
    PlayEntityAction, 0x1C, Play, ServerBound => PlayEntityActionSpec {
        data: RemainingBytes
    },
    PlaySteerVehicle, 0x1D, Play, ServerBound => PlaySteerVehicleSpec {
        data: RemainingBytes
    },
    PlaySetDisplayedRecipe, 0x1E, Play, ServerBound => PlaySetDisplayedRecipeSpec {
        data: RemainingBytes
    },
    PlaySetRecipeBookState, 0x1F, Play, ServerBound => PlaySetRecipeBookStateSpec {
        data: RemainingBytes
    },
    PlayNameItem, 0x20, Play, ServerBound => PlayNameItemSpec {
        data: RemainingBytes
    },
    PlayResourcePackStatus, 0x21, Play, ServerBound => PlayResourcePackStatusSpec {
        data: RemainingBytes
    },
    PlayAdvancementTab, 0x22, Play, ServerBound => PlayAdvancementTabSpec {
        data: RemainingBytes
    },
    PlaySelectTrade, 0x23, Play, ServerBound => PlaySelectTradeSpec {
        data: RemainingBytes
    },
    PlaySetBeaconEffect, 0x24, Play, ServerBound => PlaySetBeaconEffectSpec {
        data: RemainingBytes
    },
    PlayClientHeldItemChange, 0x25, Play, ServerBound => PlayClientHeldItemChangeSpec {
        data: RemainingBytes
    },
    PlayUpdateCommandBlock, 0x26, Play, ServerBound => PlayUpdateCommandBlockSpec {
        data: RemainingBytes
    },
    PlayUpdateCommandBlockMinecart, 0x27, Play, ServerBound => PlayUpdateCommandBlockMinecartSpec {
        data: RemainingBytes
    },
    PlayUpdateJigsawBlock, 0x28, Play, ServerBound => PlayUpdateJigsawBlockSpec {
        data: RemainingBytes
    },
    PlayCreativeInventoryAction, 0x29, Play, ServerBound => PlayCreativeInventoryActionSpec {
        data: RemainingBytes
    },
    PlayUpdateStructureBlock, 0x2A, Play, ServerBound => PlayUpdateStructureBlockSpec {
        data: RemainingBytes
    },
    PlayUpdateSign, 0x2B, Play, ServerBound => PlayUpdateSignSpec {
        data: RemainingBytes
    },
    PlayClientAnimation, 0x2C, Play, ServerBound => PlayClientAnimationSpec {
        data: RemainingBytes
    },
    PlaySpectate, 0x2D, Play, ServerBound => PlaySpectateSpec {
        data: RemainingBytes
    },
    PlayBlockPlacement, 0x2E, Play, ServerBound => PlayBlockPlacementSpec {
        data: RemainingBytes
    },
    PlayUseItem, 0x2F, Play, ServerBound => PlayUseItemSpec {
        data: RemainingBytes
    }
});

proto_struct!(LoginSuccessPropertiesSpec {
    name: String,
    value: String,
    signed: bool,
    signature: String
});

proto_byte_enum!(HandshakeNextState,
    0x01 :: Status,
    0x02 :: Login
);
