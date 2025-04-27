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

const { SlashCommandBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle, PermissionsBitField } = require('discord.js');
const { QueryType, useMainPlayer, useQueue } = require('discord-player');

module.exports = {
  category: "Music",
  cooldown: 5,
  data: new SlashCommandBuilder()
    .setName('oynat')
    .setDescription('Şarkı oynatır.')
    .addStringOption(option =>
      option.setName('şarkı')
        .setDescription('Şarkı adı veya bağlantısı')
        .setRequired(true)
    ),
  run: async (client, interaction) => {

    const player = useMainPlayer();
    const songQuery = interaction.options.getString('şarkı');

    if (songQuery.includes('youtube.com') || songQuery.includes('youtu.be'))
      return interaction.reply({
        content: '❌ | **Youtube** üzerinden şarkı oynatamıyorum. Lütfen Spotify veya SoundCloud kullanın.',
        ephemeral: true
      });

    const voiceChannel = interaction.member.voice.channel;
    if (!voiceChannel) return interaction.reply({
      content: '❌ | Ses kanalında değilsiniz.',
      ephemeral: true
    });

    if (interaction.guild.members.me.voice.channel && interaction.guild.members.me.voice.channel.id !== voiceChannel.id)
      return interaction.reply({
        content: '❌ | Başka bir ses kanalında çalıyorum!',
        ephemeral: true
      });

    if (!voiceChannel.permissionsFor(interaction.guild.members.me).has(PermissionsBitField.Flags.Connect))
      return interaction.reply({
        content: '❌ | Ses kanalına katılma iznim yok!',
        ephemeral: true
      });

    if (!voiceChannel.permissionsFor(interaction.guild.members.me).has(PermissionsBitField.Flags.Speak))
      return interaction.reply({
        content: '❌ | Ses kanalında konuşma iznim yok!',
        ephemeral: true
      });

    await interaction.deferReply({ ephemeral: true });

    const searchResult = await player.search(songQuery, {
      requestedBy: interaction.user,
      searchEngine: QueryType.AUTO
    });

    if (!searchResult || !searchResult.tracks.length)
      return interaction.editReply({
        content: '❌ | Şarkı **Spotify veya Soundcloud** üzerinde bulunamadı!'
      });

    let queue = useQueue(interaction.guild.id);
    if (!queue) queue = player.nodes.create(interaction.guild, {
      metadata: {
        channel: interaction.channel,
        requestedBy: interaction.user
      }
    });

    try {
      if (!queue.connection) await queue.connect(voiceChannel);
    } catch (error) {
      queue.delete();
      return interaction.editReply({
        content: '❌ | Ses kanalına bağlanırken hata oluştu!'
      });
    }

    const track = searchResult.tracks[0];
    queue.addTrack(track);

    if (!queue.node.isPlaying())
      try {
        await queue.node.play();
      } catch (err) {
        if (err.code === 'ERR_NO_RESULT') {
          return interaction.editReply({
            content: '❌ | Şarkı çalınamadı, kaynak bulunamadı veya erişilemiyor.'
          });
        }
        return interaction.editReply({
          content: '❌ | Şarkı oynatılırken beklenmedik bir hata oluştu.'
        });
      }

    return interaction.editReply({
      content: `🎶 **${track.title}** kuyruğa eklendi!`,
      ephemeral: true
    });

  }
};
