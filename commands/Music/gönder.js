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

const { SlashCommandBuilder, EmbedBuilder } = require('discord.js');
const { useQueue } = require('discord-player');
const MusicSetting = require("../../models/MusicSetting");

module.exports = {
  category: "Music",
  cooldown: 5,
  data: new SlashCommandBuilder()
    .setName('gönder')
    .setDescription("Oynatılan şarkıyı DM'den bilgileriyle birlikte gönderir."),

  run: async (client, interaction) => {

    let queue = useQueue(interaction.guild.id);
    if (!queue) {
      const noSong = new EmbedBuilder()
        .setColor(client.color)
        .setDescription('❌ | Şu anda hiçbir müzik oynatılmıyor.');
      return interaction.reply({ embeds: [noSong], ephemeral: true });
    }

    // ---- İzin Kontrolleri ---- //
    const voiceChannel = await checks(interaction);
    if (!voiceChannel) return;

    const track = queue.currentTrack;
    const embedDM = new EmbedBuilder()
      .setColor(client.color)
      .setTitle('🎵 Oynatılan Şarkı')
      .setImage(track.thumbnail)
      .addFields(
        { name: 'Şarkı:', value: `[${track.title}](${track.url})`, inline: false },
        { name: 'Süre:', value: track.duration, inline: true },
        { name: 'Sunucu:', value: interaction.guild.name, inline: true },
        { name: 'Oynatan:', value: `<@${track.requestedBy.id}>`, inline: true }
      )

    try {
      await interaction.user.send({ embeds: [embedDM] });
      return interaction.reply({ content: "☑️ | DM'den oynatılan şarkıyı gönderdim!", ephemeral: true });
    }

    catch (err) {
      return interaction.reply({ content: '❌ | DM gönderilemedi. Gizlilik ayarlarını kontrol et.', ephemeral: true });
    }

  }
};

async function checks(interaction) {

  const voiceChannel = interaction.member.voice.channel;
  if (!voiceChannel) {
    await interaction.reply({ content: '❌ | Ses kanalında değilsiniz.', ephemeral: true });
    return null;
  }

  const botVoice = interaction.guild.members.me.voice.channel;
  if (botVoice && botVoice.id !== voiceChannel.id) {
    await interaction.reply({ content: '❌ | Başka bir ses kanalındayım.', ephemeral: true });
    return null;
  }

  return voiceChannel;
}