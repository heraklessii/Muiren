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
const { useQueue, useTimeline, useMainPlayer } = require('discord-player');
const {
    EmbedBuilder, InteractionType, UserSelectMenuBuilder, StringSelectMenuBuilder,
    Events, ModalBuilder, TextInputBuilder, TextInputStyle, ActionRowBuilder, PermissionsBitField
} = require("discord.js");

const cooldowns = new Map();

module.exports = {
    name: Events.InteractionCreate,
    async execute(interaction) {

        if (!interaction.guild || interaction.user.bot) return;

        if (interaction.isChatInputCommand()) {
            const cmd = client.commands.get(interaction.commandName);
            if (!cmd) return;

            // cooldown
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

            const { customId } = interaction;
            const queue = useQueue(interaction.guild.id);
            const timeline = useTimeline({ node: interaction.guild.id });

            if (customId === 'pause') {

                if (!queue || !queue.isPlaying()) return interaction.reply({
                    content: '❌ | Şu anda herhangi bir şarkı oynatılmıyor.',
                    ephemeral: true
                });

                if (timeline.paused)
                    return interaction.reply({
                        content: '❌ | Şarkı zaten durdurulmuş.',
                        ephemeral: true

                    })

                timeline.pause();
                return interaction.reply({ content: '✅ | Şarkı duraklatıldı.', ephemeral: true });

            }

            else if (customId === 'resume') {

                if (!queue || !queue.isPlaying()) return interaction.reply({
                    content: '❌ | Şu anda herhangi bir şarkı oynatılmıyor.',
                    ephemeral: true
                });

                if (!timeline.paused)
                    return interaction.reply({
                        content: '❌ | Şarkı zaten devam ediyor.',
                        ephemeral: true
                    })

                timeline.resume();
                return interaction.reply({
                    content: '✅ | Şarkı devam ediyor.',
                    ephemeral: true
                });

            }

            else if (customId === 'stop') {

                if (!queue || !queue.isPlaying()) return interaction.reply({
                    content: '❌ | Şu anda herhangi bir şarkı oynatılmıyor.',
                    ephemeral: true
                });

                queue.delete();
                return interaction.reply({
                    content: '✅ | Şarkı durdu ve sıra sonlandırıldı.',
                    ephemeral: true
                });
            }

            else if (customId === 'skip') {

                if (!queue || !queue.isPlaying()) return interaction.reply({
                    content: '❌ | Şu anda herhangi bir şarkı oynatılmıyor.',
                    ephemeral: true
                });

                if (queue.tracks.size < 1)
                    return interaction.reply({
                        content: `❌ | Atlanacak şarkı bulunamadı.`,
                        ephemeral: true
                    })

                queue.node.skip()
                interaction.message.delete()
                return interaction.reply({
                    content: '✅ | Şarkı atlandı.',
                    ephemeral: true
                });

            }

        }

        // --- Select menu interactions ---
        if (interaction.isStringSelectMenu() && interaction.customId === 'ara_select') {
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