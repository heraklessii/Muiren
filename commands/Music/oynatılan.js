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
    .setName('oynatılan')
    .setDescription("Mevcut oynatılan şarkıyı gösterir."),

  run: async (client, interaction) => {

    let queue = useQueue(interaction.guild.id);
    if (!queue || !queue.node.isPlaying()) {
      const noSong = new EmbedBuilder()
        .setColor(client.color)
        .setDescription('❌ | Şu anda hiçbir müzik oynatılmıyor.');
      return interaction.reply({ embeds: [noSong], ephemeral: true });
    }

    // ---- İzin Kontrolleri ---- //
    const voiceChannel = await checks(interaction);
    if (!voiceChannel) return;

    const track = queue.currentTrack;
    const progress = queue.node.createProgressBar({ size: 45 });
    const status = queue.node.isPaused() ? "⏸️ |" : "🔴 |";

    const embed = new EmbedBuilder()
      .setColor(client.color)
      .setAuthor({
        name: queue.node.isPaused() ? 'Şarkı durduruldu...' : 'Oynatılıyor...',
        iconURL: "https://cdn.discordapp.com/emojis/741605543046807626.gif"
      })
      .setDescription(`**[${track.title}](${track.url})**`)
      .setImage(track.thumbnail)
      .addFields(
        { name: 'Oynatan Kişi:', value: `${track.requestedBy}`, inline: true },
        { name: 'Mevcut Ses:', value: `%${queue.node.volume}`, inline: true },
        { name: `Toplam Süre:`, value: `${track.duration}`, inline: true },
        {
          name: `Mevcut Süre: \`[${queue.node.getTimestamp().current.label} / ${track.duration}]\``,
          value: `\`\`\`${status} ${progress}\`\`\``,
          inline: false
        }
      )

    return interaction.reply({ embeds: [embed], ephemeral: true });

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