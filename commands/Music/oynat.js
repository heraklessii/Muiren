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

    const voiceChannel = interaction.member.voice.channel;
    if (!voiceChannel) return interaction.reply({ content: '❌ | Ses kanalında değilsiniz.', ephemeral: true });
    
    if (interaction.guild.members.me.voice.channel && interaction.guild.members.me.voice.channel.id !== voiceChannel.id)
      return interaction.reply({ content: '❌ | Başka bir ses kanalındayım.', ephemeral: true });
    
    if (!voiceChannel.permissionsFor(interaction.guild.members.me).has(PermissionsBitField.Flags.Connect))
      return interaction.reply({ content: '❌ | Kanala bağlanma iznim yok.', ephemeral: true });
    
    if (!voiceChannel.permissionsFor(interaction.guild.members.me).has(PermissionsBitField.Flags.Speak))
      return interaction.reply({ content: '❌ | Konuşma iznim yok.', ephemeral: true });

    // ✅ Tek deferReply, tek cevap:
    await interaction.deferReply({ ephemeral: true });

    // Arama
    const searchResult = await player.search(query, {
      requestedBy: interaction.user,
      searchEngine: QueryType.AUTO
    });
    if (!searchResult.tracks.length) {
      return interaction.editReply({ content: '❌ | Hiç parça bulunamadı.' });
    }

    // Kuyruk oluştur/çek
    let queue = useQueue(interaction.guild.id);
    if (!queue) {
      queue = player.nodes.create(interaction.guild, {
        metadata: { channel: interaction.channel, requestedBy: interaction.user }
      });
    }

    // Bağlan
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
