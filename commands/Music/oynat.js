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

const { SlashCommandBuilder, PermissionsBitField } = require('discord.js');
const { QueryType, useMainPlayer, useQueue } = require('discord-player');

module.exports = {
  category: "Music",
  cooldown: 5,
  data: new SlashCommandBuilder()
    .setName('oynat')
    .setDescription('Şarkı veya playlist oynatır.')
    .addStringOption(opt =>
      opt.setName('şarkı')
        .setDescription('Şarkı/playlist adı veya bağlantısı')
        .setRequired(true)),

  run: async (client, interaction) => {

    const player = useMainPlayer();
    const query = interaction.options.getString('şarkı');

    // ---- İzin Kontrolleri ---- //
    const voiceChannel = await checks(interaction);
    if (!voiceChannel) return;

    await interaction.deferReply({ ephemeral: true });

    const searchResult = await player.search(query, {
      requestedBy: interaction.user,
      searchEngine: QueryType.AUTO
    });

    if (!searchResult.tracks.length) {
      return interaction.editReply({ content: '❌ | Hiç parça bulunamadı.' });
    }

    let queue = useQueue(interaction.guild.id);
    if (!queue) {
      queue = player.nodes.create(interaction.guild, {
        metadata: { channel: interaction.channel, requestedBy: interaction.user }
      });
    }

    try {
      if (!queue.connection) await queue.connect(voiceChannel);
    } catch {
      queue.delete();
      return interaction.editReply({ content: '❌ | Ses kanalına bağlanılamadı.' });
    }

    if (searchResult.playlist) {
      queue.addTrack(searchResult.tracks);
      if (!queue.node.isPlaying()) {
        try {
          await queue.node.play();
        } catch (err) {
          const msg = err.code === 'ERR_NO_RESULT'
            ? '❌ | Parça oynatılamadı, bulunamadı veya erişilemiyor.'
            : '❌ | Oynatılırken bir hata oluştu.';
          return interaction.editReply({ content: msg });
        }
      }
      return interaction.editReply({
        content: `🎶 **${searchResult.tracks.length}** parçalık playlist kuyruğa eklendi!`
      });
    }

    else {
      // Sadece ilk parçayı ekle ve çal
      const track = searchResult.tracks[0];
      queue.addTrack(track);
      if (!queue.node.isPlaying()) {
        try {
          await queue.node.play();
        } catch (err) {
          const msg = err.code === 'ERR_NO_RESULT'
            ? '❌ | Parça oynatılamadı, bulunamadı veya erişilemiyor.'
            : '❌ | Oynatılırken bir hata oluştu.';
          return interaction.editReply({ content: msg });
        }
      }
      return interaction.editReply({
        content: `🎶 **${track.title}** kuyruğa eklendi!`
      });
    }

  }
};

async function checks(interaction) {

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