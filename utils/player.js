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

    player.events.on('volumeChange', async (oldVolume, newVolume) => {

    })

    // Kanal boşsa otomatik ayrılma
    player.events.on('emptyChannel', async (queue) => {

        const setting = await MusicSetting.findOne({ guildId: queue.guild.id });
        if (setting && setting.systemEnabled) UpdateMusic(queue);

        const Embed = new EmbedBuilder()
            .setColor(client.color)
            .setDescription(`🎵 | Kanalda tek başıma kaldım! Ayrılıyorum.`)

        return queue.metadata.channel.send({ embeds: [Embed] });

    });

    // Kuyruk sona erdiğinde, 1 dakika içinde yeni parça eklenmezse kanaldan çıkma
    player.events.on('emptyQueue', (queue) => {
        setTimeout(() => {
            if (!queue.isPlaying) {
                const Embed = new EmbedBuilder()
                    .setColor(client.color)
                    .setDescription(`⏳ | 1 dakika boyunca oynatma yok, kanaldan ayrılıyorum.`)

                queue.metadata.channel.send({ embeds: [Embed] });
                queue.connection?.disconnect();
            }
        }, 60000);
    });

    player.events.on(GuildQueueEvent.PlayerFinish, async (queue, track) => {

        const setting = await MusicSetting.findOne({ guildId: queue.guild.id });
        if (!setting || !setting.systemEnabled) {

            queue.delete();
            const embed = new EmbedBuilder()
                .setDescription(`:musical_note: | Listedeki bütün şarkıları oynatmayı bitirdim.`)
                .setColor(client.green)

            return queue.metadata.channel.send({ embeds: [embed] }).then((sent) => {
                setTimeout(() => {
                    sent.delete();
                }, 5000);
            });
        }

        else {

            UpdateMusic(queue)
            const embed = new EmbedBuilder()
                .setDescription(`:musical_note: | Listedeki bütün şarkıları oynatmayı bitirdim.`)
                .setColor(client.green)

            return queue.textChannel.send({ embeds: [embed] }).then((sent) => {
                setTimeout(() => {
                    sent.delete();
                }, 5000);
            });
        }

    });

    // Error
    player.events.on('error', (queue, error) => {
        console.log(`General player error event: ${error.message}`);
        console.log(error);
    });

    player.events.on('playerError', (queue, error) => {
        console.log(`Player error event: ${error.message}`);
        console.log(error);
    });

}
