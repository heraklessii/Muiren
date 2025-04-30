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
const { GuildQueueEvent, useTimeline, useMainPlayer, QueueRepeatMode, useHistory } = require('discord-player');
const { UpdateQueueMsg, UpdateMusic } = require("../utils/musicUpdater");
const MusicSetting = require('../models/MusicSetting');

module.exports = async (client) => {

    const player = useMainPlayer();

    player.events.on('playerStart', async (queue, track) => {

        UpdateQueueMsg(queue)

        const setting = await MusicSetting.findOne({ guildId: queue.guild.id });
        if (!setting || !setting.systemEnabled || !setting.channelId || !setting.messageId) {

            const tracksArray = typeof queue.tracks.toArray === 'function'
                ? queue.tracks.toArray()
                : queue.tracks;

            const list = tracksArray
                .map((t, i) => `*\`${i + 1} • ${t.title} • [${t.duration}]\`* • ${t.requestedBy}`)
                .slice(0, 5)
                .join('\n') || 'Sırada başka şarkı yok.';

            const track = queue.currentTrack;
            const progress = queue.node.createProgressBar({ size: 45 });
            const status = queue.node.isPaused() ? "⏸️ |" : "🔴 |";

            const Embed = new EmbedBuilder()
                .setAuthor({ name: `Şarkı oynatılıyor...`, iconURL: 'https://cdn.discordapp.com/emojis/741605543046807626.gif' })
                .setImage(track.thumbnail)
                .setColor(client.color)
                .setDescription(`**[${track.title}](${track.url})**`)
                .addFields({ name: `Oynatan Kişi:`, value: `${track.requestedBy}`, inline: true })
                .addFields({ name: `Mevcut Ses:`, value: `**%${queue.node.volume}**`, inline: true })
                .addFields({ name: `Toplam Süre:`, value: `${track.duration}`, inline: true })
                .addFields({
                    name: `Mevcut Süre: \`[${queue.node.getTimestamp().current.label} / ${track.duration}]\``,
                    value: `\`\`\`${status} ${progress}\`\`\``,
                    inline: false
                })

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

            const nowplay = await queue.metadata.channel.send({ content: `**__Şarkı Listesi:__**\n${list}`, embeds: [Embed], components: [row] })
            const collector = nowplay.createMessageComponentCollector({ time: track.durationMS });
            queue.metadata.nowplayMessage = nowplay;

            collector.on('collect', async (interaction) => {

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
                    }

                    if (timeline.paused) {

                        timeline.resume();
                        const embed = new EmbedBuilder()
                            .setColor(client.color)
                            .setDescription(`⏯ | Şarkı devam ettiriliyor.`);

                        interaction.reply({ embeds: [embed], ephemeral: true })

                    }

                    else {

                        timeline.pause();
                        const embed = new EmbedBuilder()
                            .setColor(client.color)
                            .setDescription(`⏯ | Şarkı durduruluyor.`);

                        interaction.reply({ embeds: [embed], ephemeral: true })

                    }

                }

                else if (id === "skip") {

                    if (!queue) {
                        collector.stop();
                        queue.connection?.disconnect();
                    }

                    if (queue.tracks.size < 1) {

                        if (queue.repeatMode === 3) {

                            const embed = new EmbedBuilder()
                                .setColor(client.color)
                                .setDescription("⏭ | Şarkı başarıyla atlandı.")

                            const nowplay = queue.metadata.nowplayMessage;
                            if (nowplay) queue.metadata.nowplayMessage = null;
                            queue.node.skip()
                            nowplay.delete();
                            collector.stop();
                            return interaction.reply({ embeds: [embed], ephemeral: true })

                        }

                        const embed = new EmbedBuilder()
                            .setColor(client.color)
                            .setDescription(":x: | Sırada atlanacak hiçbir şarkı yok.")

                        return interaction.reply({ embeds: [embed], ephemeral: true })

                    }

                    if (queue.repeatMode === 1) {

                        const embed = new EmbedBuilder()
                            .setColor(client.color)
                            .setDescription(":x: | Tekrar modu açık olduğu için atlayamıyorum.")

                        return interaction.reply({ embeds: [embed], ephemeral: true })

                    }

                    const embed = new EmbedBuilder()
                        .setColor(client.color)
                        .setDescription("⏭ | Şarkı başarıyla atlandı.")

                    const nowplay = queue.metadata.nowplayMessage;
                    if (nowplay) queue.metadata.nowplayMessage = null;
                    queue.node.skip()
                    nowplay.delete();
                    collector.stop();
                    interaction.reply({ embeds: [embed], ephemeral: true })

                }

                else if (id === "stop") {

                    const embed = new EmbedBuilder()
                        .setDescription(`✅ | Şarkı oynatmayı bitirdim ve kanaldan ayrıldım.`)
                        .setColor(client.color);

                    queue.delete();
                    nowplay.delete();
                    collector.stop();
                    interaction.reply({ embeds: [embed], ephemeral: true })

                }

                else if (id === "loop") {

                    if (!queue) {
                        collector.stop();
                        queue.connection?.disconnect();
                    }

                    if (queue.repeatMode === 0) {

                        queue.setRepeatMode(1);
                        const embed = new EmbedBuilder()
                            .setColor(client.color)
                            .setDescription(`🔁 | **Şarkı tekrar modu** aktif edildi.`)

                        interaction.reply({ embeds: [embed], ephemeral: true })

                    }

                    else if (queue.repeatMode === 1) {

                        if (queue.tracks.size < 1) {

                            queue.setRepeatMode(3);
                            const embed = new EmbedBuilder()
                                .setColor(client.color)
                                .setDescription(`🔁 | **Otomatik oynatma** açıldı.`)

                            return interaction.reply({ embeds: [embed], ephemeral: true })

                        }

                        queue.setRepeatMode(2);
                        const embed = new EmbedBuilder()
                            .setColor(client.color)
                            .setDescription(`🔁 | **Liste tekrar modu** açıldı.`)

                        interaction.reply({ embeds: [embed], ephemeral: true })

                    }

                    else if (queue.repeatMode === 2) {

                        queue.setRepeatMode(3);
                        const embed = new EmbedBuilder()
                            .setColor(client.color)
                            .setDescription(`🔁 | **Otomatik oynatma** açıldı.`)

                        return interaction.reply({ embeds: [embed], ephemeral: true })

                    }

                    else if (queue.repeatMode === 3) {

                        queue.setRepeatMode(0);
                        const embed = new EmbedBuilder()
                            .setColor(client.color)
                            .setDescription(`🔁 | **Tekrar modu** kapatıldı.`)

                        return interaction.reply({ embeds: [embed], ephemeral: true })
                    }

                }

                else if (id === "previous") {

                    if (!queue) {
                        collector.stop();
                        queue.connection?.disconnect();
                    }

                    const history = useHistory(interaction.guild.id);
                    if (history.disabled || history.getSize() === 0) {
                        const embed = new EmbedBuilder()
                            .setColor(client.color)
                            .setDescription(":x: | Oynatılacak eski bir şarkı bulunamadı!")

                        return interaction.reply({ embeds: [embed], ephemeral: true })
                    }

                    else {

                        const embed = new EmbedBuilder()
                            .setColor(client.color)
                            .setDescription("⏮ | Eski şarkı oynatılıyor.")

                        await history.previous();
                        nowplay.delete();
                        collector.stop();
                        interaction.reply({ embeds: [embed], ephemeral: true })

                    }
                }

            });

            collector.on('end', (_, reason) => {

                if (queue.metadata.nowplayMessage) {
                    queue.metadata.nowplayMessage = null;
                    nowplay.delete().catch(() => { });
                }

                queue?.delete();

            });

        }

    });

    player.events.on('audioTrackAdd', async (queue, track) => {

        const nowplay = queue.metadata.nowplayMessage;
        if (nowplay) update(queue, nowplay)
        UpdateQueueMsg(queue)

        const setting = await MusicSetting.findOne({ guildId: queue.guild.id });
        if (!setting || !setting.systemEnabled) {
            const embed = new EmbedBuilder()
                .setColor(client.color)
                .setDescription(`✅ | **[${track.title}](${track.url})** \`${track.duration}\` • ${track.requestedBy}`)

            return queue.metadata.channel.send({ embeds: [embed] }).then((sent) => {
                setTimeout(() => {
                    sent.delete();
                }, 5000);
            });
        }

    });

    player.events.on('audioTracksAdd', async (queue, track) => {
        const nowplay = queue.metadata.nowplayMessage;
        if (nowplay) update(queue, nowplay)
        return UpdateQueueMsg(queue);
    });

    // Kanal boşsa otomatik ayrılma
    player.events.on('emptyChannel', async (queue) => {
        const nowplay = queue.metadata.nowplayMessage;
        if (nowplay) queue.metadata.nowplayMessage = null;
        UpdateMusic(queue)
        const Embed = new EmbedBuilder()
            .setColor(client.color)
            .setDescription(`🎵 | Kanalda tek başıma kaldım! Ayrılıyorum.`)

        return queue.metadata.channel.send({ embeds: [Embed] }).then(sent => {
            setTimeout(() => sent.delete().catch(() => { }), 3000);
        });

    });

    // Kuyruk sona erdiğinde, 1 dakika içinde yeni parça eklenmezse kanaldan çıkma
    player.events.on('emptyQueue', (queue) => {
        const nowplay = queue.metadata.nowplayMessage;
        if (nowplay) queue.metadata.nowplayMessage = null;
        UpdateMusic(queue)
        setTimeout(() => {
            if (!queue.node.isPlaying()) {
                const Embed = new EmbedBuilder()
                    .setColor(client.color)
                    .setDescription(`⏳ | 1 dakika boyunca oynatma yok, kanaldan ayrılıyorum.`)

                return queue.metadata.channel.send({ embeds: [Embed] }).then(sent => {
                    setTimeout(() => sent.delete().catch(() => { }), 3000);
                });
            }
        }, 60000);
    });

    player.events.on('disconnect', (queue) => {
        const nowplay = queue.metadata.nowplayMessage;
        if (nowplay) queue.metadata.nowplayMessage = null;
        return UpdateMusic(queue)
    })

    player.events.on('playerResume', (queue) => {
        const nowplay = queue.metadata.nowplayMessage;
        if (nowplay) update(queue, nowplay)
        return UpdateQueueMsg(queue)
    })

    player.events.on('connectionDestroyed', (queue) => {
        const nowplay = queue.metadata.nowplayMessage;
        if (nowplay) queue.metadata.nowplayMessage = null;
        return UpdateMusic(queue)
    })

    player.events.on('playerPause', (queue) => {
        const nowplay = queue.metadata.nowplayMessage;
        if (nowplay) update(queue, nowplay)
        return UpdateQueueMsg(queue);
    })

    player.events.on('playerError', (queue, error) => {

        const Embed = new EmbedBuilder()
            .setColor(client.color)
            .setDescription(`🎵 | Bir hata meydana geldi! Lütfen tekrar deneyin.`)

        queue.delete();
        queue.connection?.disconnect();
        console.log(error);
        return queue.metadata.channel.send({ embeds: [Embed] }).then(sent => {
            setTimeout(() => sent.delete().catch(() => { }), 5000);
        });

    });

}

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

async function update(queue) {

    const nowplay = queue.metadata.nowplayMessage;
    if (!nowplay) return;

    const tracksArray = typeof queue.tracks.toArray === 'function'
        ? queue.tracks.toArray()
        : queue.tracks;

    const list = tracksArray
        .map((t, i) => `*\`${i + 1} • ${t.title} • [${t.duration}]\`* • ${t.requestedBy}`)
        .slice(0, 5)
        .join('\n') || 'Sırada başka şarkı yok.';

    const track = queue.currentTrack;
    const progress = queue.node.createProgressBar({ size: 45 });
    const status = queue.node.isPaused() ? "⏸️ |" : "🔴 |";

    const Embed = new EmbedBuilder()
        .setAuthor({
            name: queue.node.isPaused() ? 'Şarkı durduruldu...' : 'Oynatılıyor...',
            iconURL: "https://cdn.discordapp.com/emojis/741605543046807626.gif"
        })
        .setImage(track.thumbnail)
        .setColor(client.color)
        .setDescription(`**[${track.title}](${track.url})**`)
        .addFields({ name: `Oynatan Kişi:`, value: `${track.requestedBy}`, inline: true })
        .addFields({ name: `Mevcut Ses:`, value: `**%${queue.node.volume}**`, inline: true })
        .addFields({ name: `Toplam Süre:`, value: `${track.duration}`, inline: true })
        .addFields({
            name: `Mevcut Süre: \`[${queue.node.getTimestamp().current.label} / ${track.duration}]\``,
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

    return nowplay.edit({ content: `**__Şarkı Listesi:__**\n${list}`, embeds: [Embed], components: [row] });
}
