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

const { EmbedBuilder, ButtonBuilder, ActionRowBuilder, ButtonStyle } = require("discord.js");
const { GuildQueueEvent, useTimeline, useMainPlayer, QueueRepeatMode, useHistory } = require('discord-player');
const { UpdateQueueMsg, UpdateMusic } = require("../utils/musicUpdater");
const MusicSetting = require('../models/MusicSetting');

module.exports = async (client) => {

    const player = useMainPlayer();

    player.events.on('playerStart', async (queue, track) => {

        UpdateQueueMsg(queue)

        const setting = await MusicSetting.findOne({ guildId: queue.guild.id });
        if (!setting || !setting.systemEnabled) {

            const Embed = new EmbedBuilder()
                .setAuthor({ name: `Şarkı oynatılıyor...`, iconURL: 'https://cdn.discordapp.com/emojis/741605543046807626.gif' })
                .setImage(track.thumbnail)
                .setColor(client.color)
                .setDescription(`**[${track.title}](${track.url})**`)
                .addFields({ name: `Oynatan Kişi:`, value: `${track.requestedBy}`, inline: true })
                .addFields({ name: `Mevcut Ses:`, value: `**%${queue.node.volume}**`, inline: true })
                .addFields({ name: `Toplam Süre:`, value: `${track.duration}`, inline: true })
                .addFields({ name: `Mevcut Süre: \`[0:00 / ${track.duration}]\``, value: `\`\`\`🔴 | 🎶 ───────────────────────────────\`\`\``, inline: false })

            const row = new ActionRowBuilder().addComponents(
                new ButtonBuilder()
                    .setCustomId('pause')
                    .setEmoji('⏯️')
                    .setStyle(ButtonStyle.Success),
                new ButtonBuilder()
                    .setCustomId('previous')
                    .setEmoji('⬅️')
                    .setStyle(ButtonStyle.Primary),
                new ButtonBuilder()
                    .setCustomId('stop')
                    .setEmoji('⏹')
                    .setStyle(ButtonStyle.Danger),
                new ButtonBuilder()
                    .setCustomId('skip')
                    .setEmoji('➡️')
                    .setStyle(ButtonStyle.Primary),
                new ButtonBuilder()
                    .setCustomId('loop')
                    .setEmoji('🔄')
                    .setStyle(ButtonStyle.Success)
            );

            const nowplay = await queue.metadata.channel.send({ embeds: [Embed], components: [row] })
            const filter = (interaction) => {
                if (interaction.guild.members.me.voice.channel && interaction.guild.members.me.voice.channelId === interaction.member.voice.channelId) return true;
                else interaction.reply({ content: ":x: | Butonları kullanabilmek için benimle aynı ses kanalında olmalısın.", ephemeral: true })
            };

            const collector = nowplay.createMessageComponentCollector({ filter, time: track.durationMS });

            collector.on('collect', async (interaction) => {

                const id = interaction.customId;
                const timeline = useTimeline({ node: queue.guild.id });

                if (interaction.user.id != track.requestedBy.id) interaction.reply({
                    content: `:x: | ${interaction.user}, oynatılan şarkıyı siz eklemediğiniz için butonları kullanamazsınız.`,
                    ephemeral: true,
                })

                if (id === "pause") {

                    if (!queue) collector.stop()

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

                    if (!queue || !queue.isPlaying() || queue.tracks.size < 1) {

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

                    if (queue.repeatMode === QueueRepeatMode.OFF) {

                        queue.setRepeatMode(1);
                        const embed = new EmbedBuilder()
                            .setColor(client.color)
                            .setDescription(`🔁 | Şarkı tekrar modu aktif edildi.`)

                        interaction.reply({ embeds: [embed], ephemeral: true })

                    }

                    else {

                        queue.setRepeatMode(0);
                        const embed = new EmbedBuilder()
                            .setColor(client.color)
                            .setDescription(`🔁 | Şarkı tekrar modu kapatıldı.`)

                        interaction.reply({ embeds: [embed], ephemeral: true })

                    }

                }

                else if (id === "previous") {

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
                if (reason === "time") nowplay.delete().catch(() => { });
            });


        }

    });

    player.events.on('audioTrackAdd', async (queue, track) => {

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

    // Kanal boşsa otomatik ayrılma
    player.events.on('emptyChannel', async (queue) => {

        UpdateMusic(queue)
        const Embed = new EmbedBuilder()
            .setColor(client.color)
            .setDescription(`🎵 | Kanalda tek başıma kaldım! Ayrılıyorum.`)

        return queue.metadata.channel.send({ embeds: [Embed] });

    });

    // Kuyruk sona erdiğinde, 1 dakika içinde yeni parça eklenmezse kanaldan çıkma
    player.events.on('emptyQueue', (queue) => {
        UpdateMusic(queue)
        setTimeout(() => {
            if (!queue.node.isPlaying()) {
                const Embed = new EmbedBuilder()
                    .setColor(client.color)
                    .setDescription(`⏳ | 1 dakika boyunca oynatma yok, kanaldan ayrılıyorum.`)

                queue.connection?.disconnect();
                return queue.metadata.channel.send({ embeds: [Embed] });
            }
        }, 60000);
    });

    player.events.on('disconnect', (queue) => {
        return UpdateMusic(queue)
    })

    player.events.on(GuildQueueEvent.PlayerFinish, async (queue, track) => {

        if (queue || queue.isPlaying() || queue.tracks.size > 1) return;

        const setting = await MusicSetting.findOne({ guildId: queue.guild.id });
        if (!setting || !setting.systemEnabled) {

            const embed = new EmbedBuilder()
                .setDescription(`🎵 | Listedeki bütün şarkıları oynatmayı bitirdim.`)
                .setColor(client.color)

            return queue.metadata.channel.send({ embeds: [embed] }).then((sent) => {
                setTimeout(() => {
                    sent.delete();
                }, 5000);
            });
        }

        else {

            UpdateMusic(queue)
            const embed = new EmbedBuilder()
                .setDescription(`🎵 | Listedeki bütün şarkıları oynatmayı bitirdim.`)
                .setColor(client.color)

            return queue.metadata.channel.send({ embeds: [embed] }).then((sent) => {
                setTimeout(() => {
                    sent.delete();
                }, 5000);
            });
        }

    });

}
