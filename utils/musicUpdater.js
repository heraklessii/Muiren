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
const { ActionRowBuilder, ButtonBuilder, EmbedBuilder, ButtonStyle } = require('discord.js');
const MusicSetting = require('../models/MusicSetting');

async function UpdateQueueMsg(queue) {

    const setting = await MusicSetting.findOne({ guildId: queue.guild.id });
    if (!setting?.systemEnabled) return;

    const channel = client.channels.cache.get(setting.channelId);
    if (!channel) return;

    let msg;
    try {
        msg = await channel.messages.fetch(setting.messageId);
    } catch {
        return;
    }

    const tracksArray = typeof queue.tracks.toArray === 'function'
        ? queue.tracks.toArray()
        : queue.tracks;

    const list = tracksArray
        .map((t, i) => `*\`${i + 1} • ${t.title} • [${t.duration}]\`* • ${t.requestedBy}`)
        .slice(0, 10)
        .join('\n') || 'Sırada başka şarkı yok.';

    // Şu anki çalan parça
    const current = queue.currentTrack;
    if (!current) return UpdateMusic(queue);

    const progress = queue.node.createProgressBar({ size: 45 });
    const status = queue.node.isPaused() ? "⏸️ |" : "🔴 |";

    const embed = new EmbedBuilder()
        .setAuthor({ name: queue.node.isPlaying() ? 'Oynatılıyor...' : 'Duraklatıldı...', iconURL: 'https://cdn.discordapp.com/emojis/741605543046807626.gif' })
        .setDescription(`**[${current.title}](${current.url})** • ${current.requestedBy}`)
        .setImage(current.thumbnail)
        .addFields({
            name: `Mevcut Süre: \`[${queue.node.getTimestamp().current.label} / ${current.duration}]\``,
            value: `\`\`\`${status} ${progress}\`\`\``,
            inline: false
        })
        .setFooter({ text: `${tracksArray.length} • Kuyrukta | Ses: %${queue.node.volume}` });

    const row = new ActionRowBuilder().addComponents(
        ['spause', 'sprevious', 'sstop', 'sskip', 'sloop'].map(id =>
            new ButtonBuilder()
                .setCustomId(id)
                .setStyle(ButtonStyle.Secondary)
                .setEmoji(
                    id === 'spause' ? '⏯' :
                        id === 'sprevious' ? '⬅' :
                            id === 'sstop' ? '⏹' :
                                id === 'sskip' ? '➡' :
                                    '🔄'
                )
        )
    );

    try {
        await msg.edit({
            content: `**__Şarkı Listesi:__**\n${list}`,
            embeds: [embed],
            components: [row]
        });
    } catch { }
}

async function UpdateMusic(queue) {

    const setting = await MusicSetting.findOne({ guildId: queue.guild.id });
    if (!setting?.systemEnabled) return;

    const channel = client.channels.cache.get(setting.channelId);
    if (!channel) return;

    let msg;
    try {
        msg = await channel.messages.fetch(setting.messageId);
    } catch {
        return;
    }

    const embed = new EmbedBuilder()
        .setColor(client.color)
        .setImage(process.env.BOT_BANNER_URL)
        .setAuthor({ name: 'Oynatılmıyor...', iconURL: 'https://cdn.discordapp.com/emojis/741605543046807626.gif' })
        .setFooter({ text: `Muiren, HERA tarafından geliştirilmektedir.` })

    const row = new ActionRowBuilder().addComponents(
        ['spause', 'sprevious', 'sstop', 'sskip', 'sloop'].map(id =>
            new ButtonBuilder()
                .setCustomId(id)
                .setStyle(ButtonStyle.Secondary)
                .setEmoji(id === 'spause' ? '⏯️' : id === 'sprevious' ? '⬅️' :
                    id === 'sstop' ? '⏹' : id === 'sskip' ? '➡️' : '🔄')
                .setDisabled(true)
        )
    );

    try {
        await msg.edit({ content: '**__Şarkı Listesi:__**\nOynatılan şarkılar burada yer alacak.', embeds: [embed], components: [row] });
    } catch { }
}

async function update(queue) {

    const nowplay = queue.metadata.nowplayMessage;
    if (!nowplay) return;

    const tracksArray = (typeof queue.tracks.toArray === 'function')
        ? queue.tracks.toArray()
        : queue.tracks;

    const list = tracksArray
        .map((t, i) => `*\`${i + 1} • ${t.title} • [${t.duration}]\`* • ${t.requestedBy}`)
        .slice(0, 5)
        .join('\n') || 'Sırada başka şarkı yok.';

    const currentTrack = queue.currentTrack;
    const progress = queue.node.createProgressBar({ size: 45 });
    const status = queue.node.isPaused() ? "⏸️ |" : "🔴 |";

    const Embed = new EmbedBuilder()
        .setAuthor({
            name: queue.node.isPaused() ? 'Şarkı durduruldu...' : 'Oynatılıyor...',
            iconURL: "https://cdn.discordapp.com/emojis/741605543046807626.gif"
        })
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

    return nowplay.edit({ content: `**__Şarkı Listesi:__**\n${list}`, embeds: [Embed], components: [row] });
}

module.exports = { UpdateQueueMsg, UpdateMusic, update };
