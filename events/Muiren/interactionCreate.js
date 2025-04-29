// Muiren Bot - Discord Bot Project
// Copyright (C) 2025  Muiren Bot Contributors
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

const client = global.client;
const { useQueue, useMainPlayer, QueryType } = require('discord-player');
const {
    EmbedBuilder, InteractionType, UserSelectMenuBuilder, StringSelectMenuBuilder,
    Events, ModalBuilder, TextInputBuilder, TextInputStyle, ActionRowBuilder, PermissionsBitField
} = require("discord.js");
const MusicSetting = require("../../models/MusicSetting");
const cooldowns = new Map();
const { UpdateQueueMsg, UpdateMusic } = require("../../utils/musicUpdater");

module.exports = {
    name: Events.InteractionCreate,
    async execute(interaction) {

        if (!interaction.guild || interaction.user.bot) return;

        if (interaction.isChatInputCommand()) {

            const cmd = client.commands.get(interaction.commandName);
            if (!cmd) return;

            const now = Date.now();
            const timestamps = cooldowns.get(cmd.name) || new Map();
            cooldowns.set(cmd.name, timestamps);

            const cooldownAmount = (cmd.cooldown || 5) * 1000;
            if (timestamps.has(interaction.user.id)) {
                const expire = timestamps.get(interaction.user.id) + cooldownAmount;
                if (now < expire) {
                    const remain = ((expire - now) / 1000).toFixed(1);
                    return interaction.reply({ content: `❌ | ${remain}s bekle!`, ephemeral: true });
                }
            }

            timestamps.set(interaction.user.id, now);
            setTimeout(() => timestamps.delete(interaction.user.id), cooldownAmount);

            try {
                if (interaction.type === InteractionType.ApplicationCommand) {
                    await cmd.run(client, interaction);
                }
            } catch (err) {
                console.error(err);
                if (interaction.deferred || interaction.replied) {
                    await interaction.editReply({ content: '❌ | Komut çalıştırılırken bir hata oluştu.' });
                } else {
                    await interaction.reply({ content: '❌ | Komut çalıştırılırken bir hata oluştu.', ephemeral: true });
                }
            }
            return;
        }

        else if (interaction.isButton()) {

            const { customId, member } = interaction;

            const setting = await MusicSetting.findOne({ guildId: interaction.guild.id });
            if (!setting || !setting.systemEnabled) return;

            const queue = useQueue(interaction.guild.id);
            const userVoice = member.voice.channel;
            const djRoleID = setting?.djRoleID;

            switch (customId) {

                case 'sprevious': {

                    // ---- İzin Kontrolleri ---- //
                    const voiceChannel = await checks(interaction, queue, djRoleID);
                    if (!voiceChannel) return;

                    if (!queue.history.length)
                        return interaction.reply({
                            embeds: [
                                new EmbedBuilder()
                                    .setColor(client.color)
                                    .setDescription('❌ | Önceki şarkı yok.')
                            ], ephemeral: true
                        });

                    await queue.previous();
                    UpdateQueueMsg(queue);

                    return interaction.reply({
                        embeds: [
                            new EmbedBuilder()
                                .setColor(client.color)
                                .setDescription('⬅️ | Önceki şarkıya geçildi.')
                        ], ephemeral: true
                    });

                }

                case 'sskip': {

                    // ---- İzin Kontrolleri ---- //
                    const check = await checks(interaction, queue);
                    if (!check) return;

                    if (queue.tracks.size < 1)
                        return interaction.reply({
                            embeds: [
                                new EmbedBuilder()
                                    .setColor(client.color)
                                    .setDescription('❌ | Atlanacak şarkı yok.')
                            ], ephemeral: true
                        });

                    if (queue.repeatMode === 1)
                        return interaction.reply({
                            embeds: [
                                new EmbedBuilder()
                                    .setColor(client.color)
                                    .setDescription("❌ | Tekrar modu açık olduğu için atlayamıyorum.")
                            ], ephemeral: true
                        });

                    queue.node.skip();
                    UpdateQueueMsg(queue);

                    return interaction.reply({
                        embeds: [
                            new EmbedBuilder()
                                .setColor(client.color)
                                .setDescription('➡️ | Şarkı atlandı.')
                        ], ephemeral: true
                    });

                }

                case 'sstop': {

                    // ---- İzin Kontrolleri ---- //
                    const voiceChannel = await checks(interaction, queue);
                    if (!voiceChannel) return;

                    queue.delete();
                    queue.connection?.disconnect();
                    UpdateMusic(queue);

                    return interaction.reply({
                        embeds: [
                            new EmbedBuilder()
                                .setColor(client.color)
                                .setDescription(`✅ | ${userVoice} kanalından ayrıldım.`)
                        ], ephemeral: true
                    });

                }

                case 'spause': {

                    // ---- İzin Kontrolleri ---- //
                    const voiceChannel = await checks(interaction, queue);
                    if (!voiceChannel) return;

                    if (queue.node.isPaused()) {

                        queue.node.resume();
                        return interaction.reply({
                            embeds: [
                                new EmbedBuilder()
                                    .setColor(client.color)
                                    .setDescription('⏯️ | Devam ettirildi.')
                            ], ephemeral: true
                        });
                    }

                    else {

                        queue.node.pause();
                        return interaction.reply({
                            embeds: [
                                new EmbedBuilder()
                                    .setColor(client.color)
                                    .setDescription('⏯️ | Duraklatıldı.')
                            ], ephemeral: true
                        });
                    }

                }

                case 'sloop': {

                    // ---- İzin Kontrolleri ---- //
                    const voiceChannel = await checks(interaction, queue);
                    if (!voiceChannel) return;

                    const mode = queue.repeatMode;
                    if (mode === 0) {

                        queue.setRepeatMode(1);
                        return interaction.reply({
                            embeds: [new EmbedBuilder()
                                .setColor('Green')
                                .setDescription('🔁 | Tekrar modu açıldı.')
                            ], ephemeral: true
                        });
                    }

                    else if (mode === 1) {

                        queue.setRepeatMode(0);
                        return interaction.reply({
                            embeds: [
                                new EmbedBuilder()
                                    .setColor('Green')
                                    .setDescription('🔁 | Tekrar modu kapatıldı.')
                            ], ephemeral: true
                        });
                    }

                }

                default:
                    return;

            }
        }

        else if (interaction.isStringSelectMenu()) {

            if (interaction.customId === 'ara_select') {

                const trackUrl = interaction.values[0];
                const guildId = interaction.guild.id;
                const channel = interaction.channel;
                const player = useMainPlayer();

                const searchResult = await player.search(trackUrl, {
                    requestedBy: interaction.user,
                    searchEngine: QueryType.AUTO
                });
                if (!searchResult.tracks.length) {
                    return interaction.reply({ content: '❌ | Şarkı bulunamadı!', ephemeral: true });
                }

                let queue = useQueue(guildId);
                if (!queue) {
                    queue = player.nodes.create(interaction.guild, {
                        metadata: { channel, requestedBy: interaction.user },
                        leaveOnEnd: true,
                        leaveOnEndCooldown: 60000
                    });
                }

                // ---- İzin Kontrolleri ---- //
                const voiceChannel = await checks2(interaction);
                if (!voiceChannel) return;

                const track = searchResult.tracks[0];

                try {
                    if (!queue.connection) await queue.connect(voiceChannel);
                    queue.addTrack(track);
                    if (!queue.node.isPlaying()) await queue.node.play();
                    return interaction.reply({ content: '🎶 | Şarkı sıraya eklendi!', ephemeral: true });
                } catch (err) {
                    queue.delete();
                    return interaction.reply({ content: '❌ | Şarkı çalınırken hata oluştu.', ephemeral: true });
                }

            }

        }

    }
};

async function checks(interaction, queue, djRoleID) {

    const voiceChannel = interaction.member.voice.channel;
    if (!voiceChannel) {
        await interaction.reply({ content: '❌ | Ses kanalında değilsiniz.', ephemeral: true });
        return null;
    }

    const permissions = voiceChannel.permissionsFor(interaction.guild.members.me);
    if (!permissions.has(PermissionsBitField.Flags.Connect)) {
        await interaction.reply({ content: '❌ | Kanala bağlanma iznim yok.', ephemeral: true });
        return null;
    }

    const botVoice = interaction.guild.members.me.voice.channel;
    if (botVoice && botVoice.id !== voiceChannel.id) {
        await interaction.reply({ content: '❌ | Başka bir ses kanalındayım.', ephemeral: true });
        return null;
    }

    if (!permissions.has(PermissionsBitField.Flags.Speak)) {
        await interaction.reply({ content: '❌ | Konuşma iznim olmadığı için şarkı oynatamıyorum.', ephemeral: true });
        return null;
    }

    // Kullanıcıda DJ rolü varsa kontrolden muaf olur.
    const isDJ = djRoleID && interaction.member.roles.cache.has(djRoleID);
    const track = queue.currentTrack;
    if (!isDJ && track?.requestedBy?.id !== interaction.user.id) {
        await interaction.reply({ content: '❌ | Bu şarkıyı sen açmadın, butonları kullanamazsın.', ephemeral: true });
        return null;
    }

    return voiceChannel;
}

async function checks2(interaction) {

    const voiceChannel = interaction.member.voice.channel;
    if (!voiceChannel) {
        await interaction.reply({ content: '❌ | Ses kanalında değilsiniz.', ephemeral: true });
        return null;
    }

    const permissions = voiceChannel.permissionsFor(interaction.guild.members.me);
    if (!permissions.has(PermissionsBitField.Flags.Connect)) {
        await interaction.reply({ content: '❌ | Kanala bağlanma iznim yok.', ephemeral: true });
        return null;
    }

    const botVoice = interaction.guild.members.me.voice.channel;
    if (botVoice && botVoice.id !== voiceChannel.id) {
        await interaction.reply({ content: '❌ | Başka bir ses kanalındayım.', ephemeral: true });
        return null;
    }

    if (!permissions.has(PermissionsBitField.Flags.Speak)) {
        await interaction.reply({ content: '❌ | Konuşma iznim olmadığı için şarkı oynatamıyorum.', ephemeral: true });
        return null;
    }

    return voiceChannel;
}