// Tsumi Bot - Discord Bot Project
// Copyright (C) 2025  Tsumi Bot Contributors
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
        .map((t, i) => `*\`${i + 1} • ${t.title} • [${t.duration}]\`* • ${t.requestedBy.username}`)
        .slice(0, 10)
        .join('\n') || 'Sırada başka şarkı yok.';

    // Şu anki çalan parça
    const current = queue.currentTrack;
    if (!current) return UpdateMusic(queue);

    const embed = new EmbedBuilder()
        .setAuthor({ name: queue.node.isPlaying() ? 'Oynatılıyor...' : 'Duraklatıldı...', iconURL: 'https://cdn.discordapp.com/emojis/741605543046807626.gif' })
        .setDescription(`**[${current.title}](${current.url})** \`[${current.duration}]\` • ${current.requestedBy}`)
        .setImage(current.thumbnail)
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
            content: `**__Listesi:__**\n${list}`,
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
        .setAuthor({ name: 'Oynatılmıyor...', iconURL: 'https://cdn.discordapp.com/emojis/741605543046807626.gif' })
        .setImage(process.env.BOT_BANNER_URL)
        .setFooter({ text: `0 • Kuyrukta | Ses: %${queue.node.volume}` })
        .setColor(client.color);


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
                .setDisabled(true)
        )
    );

    try {
        await msg.edit({ content: '**__Liste:__**\n', embeds: [embed], components: [row] });
    } catch { }
}

module.exports = { UpdateQueueMsg, UpdateMusic };
