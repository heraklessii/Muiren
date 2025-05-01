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

const { EmbedBuilder, ButtonBuilder, ActionRowBuilder, ButtonStyle, PermissionsBitField } = require("discord.js");
const { useTimeline, useMainPlayer, useHistory } = require('discord-player');
const { UpdateQueueMsg, UpdateMusic, update } = require("../utils/musicUpdater");
const MusicSetting = require('../models/MusicSetting');

module.exports = async (client) => {

    const player = useMainPlayer();

    // Event: Şarkı Başlatıldığında (playerStart)
    // ----------------------------------
    player.events.on('playerStart', async (queue, track) => {

        UpdateQueueMsg(queue);

        const setting = await MusicSetting.findOne({ guildId: queue.guild.id });
        if (!setting || !setting.systemEnabled || !setting.channelId || !setting.messageId) {

            const tracksArray = (typeof queue.tracks.toArray === 'function')
                ? queue.tracks.toArray()
                : queue.tracks;

            const list = tracksArray
                .map((t, i) => `*\`${i + 1} • ${t.title} • [${t.duration}]\`* • ${t.requestedBy.username}`)
                .slice(0, 5)
                .join('\n') || 'Sırada başka şarkı yok.';

            const currentTrack = queue.currentTrack;
            const progress = queue.node.createProgressBar({ size: 45 });
            const status = queue.node.isPaused() ? "⏸️ |" : "🔴 |";

            const Embed = new EmbedBuilder()
                .setAuthor({ name: `Şarkı oynatılıyor...`, iconURL: 'https://cdn.discordapp.com/emojis/741605543046807626.gif' })
                .setImage(currentTrack.thumbnail)
                .setColor(client.color)
                .setDescription(`**[${currentTrack.title}](${currentTrack.url})**`)
                .addFields({ name: `Oynatan Kişi:`, value: `${currentTrack.requestedBy}`, inline: true })
                .addFields({ name: `Mevcut Ses:`, value: `**%${queue.node.volume}**`, inline: true })
                .addFields({ name: `Toplam Süre:`, value: `${currentTrack.duration}`, inline: true })
                .addFields({
                    name: `Mevcut Süre: \`[${queue.node.getTimestamp().current.label} / ${currentTrack.duration}]\``,
                    value: `\`\`\`${status} ${progress}\`\`\``,
                    inline: false
                });

            const row = new ActionRowBuilder().addComponents(
                new ButtonBuilder()
                    .setCustomId('pause')
                    .setEmoji('⏯️')
                    .setStyle(ButtonStyle.Secondary),
                new ButtonBuilder()
                    .setCustomId('previous')
                    .setEmoji('⬅️')
                    .setStyle(ButtonStyle.Secondary),
                new ButtonBuilder()
                    .setCustomId('stop')
                    .setEmoji('⏹')
                    .setStyle(ButtonStyle.Secondary),
                new ButtonBuilder()
                    .setCustomId('skip')
                    .setEmoji('➡️')
                    .setStyle(ButtonStyle.Secondary),
                new ButtonBuilder()
                    .setCustomId('loop')
                    .setEmoji('🔄')
                    .setStyle(ButtonStyle.Secondary)
            );

            const nowplay = await queue.metadata.channel.send({ content: `**__Şarkı Listesi:__**\n${list}`, embeds: [Embed], components: [row] });
            const collector = nowplay.createMessageComponentCollector();
            queue.metadata.nowplayMessage = nowplay;
            queue.metadata.collector = collector;

            collector.on('collect', async (interaction) => {
                try {

                    const id = interaction.customId;
                    const timeline = useTimeline({ node: queue.guild.id });
                    const djRoleID = setting?.djRoleID;

                    // ---- İzin Kontrolleri ---- //
                    const voiceChannel = await checks(interaction, queue, djRoleID);
                    if (!voiceChannel) return;

                    if (id === "pause") {

                        if (!queue) {
                            collector.stop();
                            queue.connection?.disconnect();
                            return;
                        }

                        if (timeline.paused) {
                            timeline.resume();
                            const embed = new EmbedBuilder()
                                .setColor(client.color)
                                .setDescription(`⏯ | Şarkı devam ettiriliyor.`);
                            await interaction.reply({ embeds: [embed], ephemeral: true });
                        }

                        else {
                            timeline.pause();
                            const embed = new EmbedBuilder()
                                .setColor(client.color)
                                .setDescription(`⏯ | Şarkı durduruluyor.`);
                            await interaction.reply({ embeds: [embed], ephemeral: true });
                        }

                    }

                    else if (id === "skip") {

                        if (!queue) {
                            collector.stop();
                            queue.connection?.disconnect();
                            return;
                        }

                        if (queue.tracks.size < 1) {

                            if (queue.repeatMode === 3) {

                                const embed = new EmbedBuilder()
                                    .setColor(client.color)
                                    .setDescription("❌ | **Otomatik oynatma** açık olduğu için atlayamıyorum.")

                                return await interaction.reply({ embeds: [embed], ephemeral: true });
                            }

                            const embed = new EmbedBuilder()
                                .setColor(client.color)
                                .setDescription("❌ | Sırada atlanacak hiçbir şarkı yok.");

                            return await interaction.reply({ embeds: [embed], ephemeral: true });

                        }

                        if (queue.repeatMode === 1) {
                            const embed = new EmbedBuilder()
                                .setColor(client.color)
                                .setDescription(":x: | Tekrar modu açık olduğu için atlayamıyorum.");
                            return await interaction.reply({ embeds: [embed], ephemeral: true });
                        }

                        const embed = new EmbedBuilder()
                            .setColor(client.color)
                            .setDescription("⏭ | Şarkı başarıyla atlandı.");

                        if (queue.metadata.nowplayMessage) queue.metadata.nowplayMessage = null;
                        if (queue.metadata.collector) queue.metadata.collector = null;
                        queue.node.skip();
                        nowplay.delete();
                        collector.stop();
                        await interaction.reply({ embeds: [embed], ephemeral: true });
                    }

                    else if (id === "stop") {

                        const embed = new EmbedBuilder()
                            .setDescription(`✅ | Şarkı oynatmayı bitirdim ve kanaldan ayrıldım.`)
                            .setColor(client.color);

                        queue.delete();
                        nowplay.delete();
                        collector.stop();
                        await interaction.reply({ embeds: [embed], ephemeral: true });
                    }

                    else if (id === "loop") {

                        if (!queue) {
                            collector.stop();
                            queue.connection?.disconnect();
                            return;
                        }

                        if (queue.repeatMode === 0) {
                            queue.setRepeatMode(1);
                            const embed = new EmbedBuilder()
                                .setColor(client.color)
                                .setDescription(`🔁 | **Şarkı tekrar modu** aktif edildi.`);
                            await interaction.reply({ embeds: [embed], ephemeral: true });
                        }

                        else if (queue.repeatMode === 1) {

                            if (queue.tracks.size < 1) {
                                queue.setRepeatMode(3);
                                const embed = new EmbedBuilder()
                                    .setColor(client.color)
                                    .setDescription(`🔁 | **Otomatik oynatma** açıldı.`);
                                return await interaction.reply({ embeds: [embed], ephemeral: true });
                            }

                            queue.setRepeatMode(2);
                            const embed = new EmbedBuilder()
                                .setColor(client.color)
                                .setDescription(`🔁 | **Liste tekrar modu** açıldı.`);

                            await interaction.reply({ embeds: [embed], ephemeral: true });

                        }

                        else if (queue.repeatMode === 2) {

                            queue.setRepeatMode(3);
                            const embed = new EmbedBuilder()
                                .setColor(client.color)
                                .setDescription(`🔁 | **Otomatik oynatma** açıldı.`);
                            return await interaction.reply({ embeds: [embed], ephemeral: true });

                        }

                        else if (queue.repeatMode === 3) {

                            queue.setRepeatMode(0);
                            const embed = new EmbedBuilder()
                                .setColor(client.color)
                                .setDescription(`🔁 | **Tekrar modu** kapatıldı.`);

                            return await interaction.reply({ embeds: [embed], ephemeral: true });

                        }

                    }

                    else if (id === "previous") {

                        if (!queue) {
                            collector.stop();
                            queue.connection?.disconnect();
                            return;
                        }

                        const history = useHistory(interaction.guild.id);
                        if (history.disabled || history.getSize() === 0) {

                            const embed = new EmbedBuilder()
                                .setColor(client.color)
                                .setDescription(":x: | Oynatılacak eski bir şarkı bulunamadı!");

                            return await interaction.reply({ embeds: [embed], ephemeral: true });
                        }

                        else {

                            const embed = new EmbedBuilder()
                                .setColor(client.color)
                                .setDescription("⏮ | Eski şarkı oynatılıyor.");

                            if (queue.metadata.nowplayMessage) queue.metadata.nowplayMessage = null;
                            if (queue.metadata.collector) queue.metadata.collector = null;
                            await history.previous();
                            nowplay.delete();
                            collector.stop();
                            await interaction.reply({ embeds: [embed], ephemeral: true });
                        }

                    }

                } catch (error) { }

            });

            collector.on('end', (_, reason) => {

                if (queue.metadata.nowplayMessage) {
                    nowplay?.delete().catch(() => { });
                    queue.metadata.nowplayMessage = null;
                }

                if (queue.metadata.collector) queue.metadata.collector = null;

            });

        }

    });

    // Event: Tekli Şarkı Eklendi (audioTrackAdd)
    // ----------------------------------
    player.events.on('audioTrackAdd', async (queue, track) => {

        UpdateQueueMsg(queue);
        if (queue.metadata.nowplayMessage) await update(queue);

        const setting = await MusicSetting.findOne({ guildId: queue.guild.id });
        if (!setting || !setting.systemEnabled) {

            const embed = new EmbedBuilder()
                .setColor(client.color)
                .setDescription(`✅ | **[${track.title}](${track.url})** \`${track.duration}\` • ${track.requestedBy}`);

            return queue.metadata.channel.send({ embeds: [embed] }).then((sent) => {
                setTimeout(() => {
                    sent.delete();
                }, 5000);
            });

        }

    });

    // Event: Liste Şarkı Eklendi (audioTracksAdd)
    // ----------------------------------
    player.events.on('audioTracksAdd', async (queue, track) => {

        UpdateQueueMsg(queue);
        if (queue.metadata.nowplayMessage) await update(queue);

    });

    // Event: Kanal Boş (emptyChannel)
    // ----------------------------------
    player.events.on('emptyChannel', async (queue) => {

        UpdateMusic(queue);

        if (queue.metadata.nowplayMessage) {
            queue.metadata.nowplayMessage.delete().catch(() => { });
            queue.metadata.nowplayMessage = null;
        }

        if (queue.metadata.collector) {
            queue.metadata.collector.stop();
            queue.metadata.collector = null;
        }

        const Embed = new EmbedBuilder()
            .setColor(client.color)
            .setDescription(`🎵 | Kanalda tek başıma kaldım! Ayrılıyorum.`);

        return queue.metadata.channel.send({ embeds: [Embed] }).then(sent => {
            setTimeout(() => sent.delete().catch(() => { }), 3000);
        });

    });

    // Event: Kuyruk Bitti (emptyQueue)
    // ----------------------------------
    player.events.on('emptyQueue', (queue) => {

        UpdateMusic(queue);

        if (queue.metadata.nowplayMessage) {
            queue.metadata.nowplayMessage.delete().catch(() => { });
            queue.metadata.nowplayMessage = null;
        }

        if (queue.metadata.collector) {
            queue.metadata.collector.stop();
            queue.metadata.collector = null;
        }

        setTimeout(() => {
            if (!queue.node.isPlaying()) {

                const Embed = new EmbedBuilder()
                    .setColor(client.color)
                    .setDescription(`⏳ | 1 dakika boyunca oynatma yok, kanaldan ayrılıyorum.`);

                return queue.metadata.channel.send({ embeds: [Embed] }).then(sent => {
                    setTimeout(() => sent.delete().catch(() => { }), 3000);
                });
            }
        }, 60000);
    });

    // Event: Bağlantı Kesildi (disconnect)
    // ----------------------------------
    player.events.on('disconnect', (queue) => {

        UpdateMusic(queue);

        if (queue.metadata.nowplayMessage) {
            queue.metadata.nowplayMessage.delete().catch(() => { });
            queue.metadata.nowplayMessage = null;
        }

        if (queue.metadata.collector) {
            queue.metadata.collector.stop();
            queue.metadata.collector = null;
        }

        return;

    });

    // Event: Şarkı Devam Ettirildi (playerResume)
    // ----------------------------------
    player.events.on('playerResume', async (queue) => {

        UpdateQueueMsg(queue);
        if (queue.metadata.nowplayMessage) await update(queue);
        return;

    });

    // Event: Bağlantı Kapatıldı (connectionDestroyed)
    // ----------------------------------
    player.events.on('connectionDestroyed', (queue) => {

        UpdateMusic(queue);

        if (queue.metadata.nowplayMessage) {
            queue.metadata.nowplayMessage.delete().catch(() => { });
            queue.metadata.nowplayMessage = null;
        }

        if (queue.metadata.collector) {
            queue.metadata.collector.stop();
            queue.metadata.collector = null;
        }

        return;

    });

    // Event: Şarkı Durduruldu (playerPause)
    // ----------------------------------
    player.events.on('playerPause', async (queue) => {

        UpdateQueueMsg(queue);
        if (queue.metadata.nowplayMessage) await update(queue);
        return;

    });

    // Event: Oynatıcı Hatası (playerError)
    // ----------------------------------
    player.events.on('playerError', (queue, error) => {

        const Embed = new EmbedBuilder()
            .setColor(client.color)
            .setDescription(`🎵 | Bir hata meydana geldi! Lütfen tekrar deneyin.`);

        queue.delete();
        queue.connection?.disconnect();

        if (queue.metadata.nowplayMessage) {
            queue.metadata.nowplayMessage.delete().catch(() => { });
            queue.metadata.nowplayMessage = null;
        }
        if (queue.metadata.collector) {
            queue.metadata.collector.stop();
            queue.metadata.collector = null;
        }

        console.log(error);
        return queue.metadata.channel.send({ embeds: [Embed] }).then(sent => {
            setTimeout(() => sent.delete().catch(() => { }), 5000);
        });

    });

    // Event: Tekli Şarkı Kaldırıldı (audioTrackRemove)
    // ----------------------------------
    player.events.on('audioTrackRemove', async (queue, track) => {

        UpdateQueueMsg(queue);
        if (queue.metadata.nowplayMessage) await update(queue);

        const setting = await MusicSetting.findOne({ guildId: queue.guild.id });
        if (!setting || !setting.systemEnabled) {

            const embed = new EmbedBuilder()
                .setColor(client.color)
                .setDescription(`:x: | **[${track.title}](${track.url})** \`${track.duration}\` • ${track.requestedBy}`);

            return queue.metadata.channel.send({ embeds: [embed] }).then((sent) => {
                setTimeout(() => {
                    sent.delete();
                }, 5000);
            });

        }

    })

    // Event: Çoklı Şarkı Kaldırıldı (audioTracksRemove)
    // ----------------------------------
    player.events.on('audioTracksRemove', async (queue, track) => {

        UpdateQueueMsg(queue);
        if (queue.metadata.nowplayMessage) await update(queue);

    });

    player.events.on('playerFinish', async (queue) => {

        if (queue.metadata.nowplayMessage) {
            queue.metadata.nowplayMessage.delete().catch(() => { });
            queue.metadata.nowplayMessage = null;
        }

        if (queue.metadata.collector) {
            queue.metadata.collector.stop();
            queue.metadata.collector = null;
        }

        return;

    })

    // Fonksiyon: checks
    // ----------------------------
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

};
