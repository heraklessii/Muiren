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
const { useQueue, useMainPlayer } = require('discord-player');
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
                interaction.reply({ content: '❌ | Bir hata oluştu.', ephemeral: true });
            }
            return;
        }

        else if (interaction.isButton()) {

            const { customId, member } = interaction;

            const setting = await MusicSetting.findOne({ guildId: interaction.guild.id });
            if (!setting || !setting.systemEnabled) return;

            const queue = useQueue(interaction.guild.id);
            const userVoice = member.voice.channel;
            const botVoice = interaction.guild.members.me.voice.channel;
            if (!queue || !userVoice) return interaction.reply({ content: ':x: | Bir ses kanalında olmalısınız.', ephemeral: true });
            if (botVoice && botVoice.id !== userVoice.id) return interaction.reply({ content: ':x: | Benimle aynı kanalda olmalısınız.', ephemeral: true });

            switch (customId) {
                case 'sprevious': {
                    if (!queue.history.length) {
                        return interaction.reply({ embeds: [new EmbedBuilder().setColor('Red').setDescription('🚨 | Önceki şarkı yok.')], ephemeral: true });
                    }
                    await queue.previous();
                    UpdateQueueMsg(queue);
                    return interaction.reply({ embeds: [new EmbedBuilder().setColor('Green').setDescription('⏮ | Önceki şarkıya geçildi.')], ephemeral: true });
                }
                case 'sskip': {
                    if (queue.tracks.size < 1) {
                        return interaction.reply({ embeds: [new EmbedBuilder().setColor('Red').setDescription('🚨 | Atlanacak şarkı yok.')], ephemeral: true });
                    }
                    
                    if (queue.repeatMode === 1) {

                        const embed = new EmbedBuilder()
                            .setColor(client.color)
                            .setDescription(":x: | Tekrar modu açık olduğu için atlayamıyorum.")

                        return interaction.reply({ embeds: [embed], ephemeral: true })

                    }
                    
                    queue.node.skip();
                    UpdateQueueMsg(queue);
                    return interaction.reply({ embeds: [new EmbedBuilder().setColor('Green').setDescription('⏭ | Şarkı atlandı.')], ephemeral: true });
                }
                case 'sstop': {
                    queue.delete();
                    queue.connection?.disconnect();
                    UpdateMusic(queue);
                    return interaction.reply({ embeds: [new EmbedBuilder().setColor('Green').setDescription(`🚫 | \`${userVoice.name}\` kanalından ayrıldım.`)], ephemeral: true });
                }
                case 'spause': {
                    if (queue.node.isPaused()) {
                        queue.node.resume();
                        UpdateQueueMsg(queue);
                        return interaction.reply({ embeds: [new EmbedBuilder().setColor('Green').setDescription('⏯ | Devam ettirildi.')], ephemeral: true });
                    }
                    else {
                        queue.node.pause();
                        UpdateQueueMsg(queue);
                        return interaction.reply({ embeds: [new EmbedBuilder().setColor('Green').setDescription('⏯ | Duraklatıldı.')], ephemeral: true });
                    }
                }
                case 'sloop': {
                    const mode = queue.repeatMode;
                    if (mode === 0) {
                        queue.setRepeatMode(1);
                        return interaction.reply({ embeds: [new EmbedBuilder().setColor('Green').setDescription('🔁 | Tekrar modu açıldı.')], ephemeral: true });
                    }
                    else {
                        queue.setRepeatMode(0);
                        return interaction.reply({ embeds: [new EmbedBuilder().setColor('Green').setDescription('🔁 | Tekrar modu kapatıldı.')], ephemeral: true });
                    }
                }
                default:
                    return;
            }

        }

        // --- Select menu interactions ---
        else if (interaction.isStringSelectMenu() && interaction.customId === 'ara_select') {
            // Acknowledge
            await interaction.deferReply({ ephemeral: true });

            const trackUrl = interaction.values[0];
            const guildId = interaction.guild.id;
            const member = interaction.member;
            const channel = interaction.channel;
            const player = useMainPlayer();

            // Get or create queue
            let queue = useQueue(guildId);
            if (!queue) {
                queue = player.nodes.create(interaction.guild, {
                    metadata: { channel, requestedBy: interaction.user }
                });
            }

            // Voice channel checks
            const voiceChannel = member.voice.channel;
            if (!voiceChannel) {
                return interaction.followUp({ content: '❌ | Ses kanalında değilsiniz.', ephemeral: true });
            }

            if (!voiceChannel.permissionsFor(interaction.guild.members.me).has(PermissionsBitField.Flags.Connect)) {
                return interaction.followUp({ content: '❌ | Katılma iznim yok.', ephemeral: true });
            }

            if (!voiceChannel.permissionsFor(interaction.guild.members.me).has(PermissionsBitField.Flags.Speak)) {
                return interaction.followUp({ content: '❌ | Konuşma iznim yok.', ephemeral: true });
            }

            // Add track and play
            try {
                if (!queue.connection) await queue.connect(voiceChannel);
                queue.addTrack(trackUrl);
                if (!queue.node.isPlaying()) await queue.node.play();
                return interaction.followUp({ content: '🎶 Şarkı oynatılıyor!', ephemeral: true });
            } catch (err) {
                console.error(err);
                queue.delete();
                return interaction.followUp({ content: '❌ | Şarkı çalınırken hata oluştu.', ephemeral: true });
            }
        }

    }
};