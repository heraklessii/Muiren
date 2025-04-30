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
    .setName('oto-oynat')
    .setDescription('Otomatik oynatma modunu açar/kapatır.'),

  run: async (client, interaction) => {

    let queue = useQueue(interaction.guild.id);
    if (!queue || !queue.node.isPlaying()) {
      const noSong = new EmbedBuilder()
        .setColor(client.color)
        .setDescription('❌ | Şu anda hiçbir müzik oynatılmıyor.');
      return interaction.reply({ embeds: [noSong], ephemeral: true });
    }

    const setting = await MusicSetting.findOne({ guildId: interaction.guild.id });
    const djRoleID = setting?.djRoleID;

    // ---- İzin Kontrolleri ---- //
    const voiceChannel = await checks(interaction, djRoleID);
    if (!voiceChannel) return;

    if (queue.repeatMode === 0 || queue.repeatMode === 1 || queue.repeatMode === 2) {

      queue.setRepeatMode(3)
      const embed = new EmbedBuilder()
        .setColor(client.color)
        .setDescription(`☑️ | Oto Oynatma: \`Aktif\``);

      return interaction.reply({ embeds: [embed] });

    }

    else {

      queue.setRepeatMode(0)
      const embed = new EmbedBuilder()
        .setColor(client.color)
        .setDescription(`☑️ | Oto Oynatma: \`Kapalı\``);

      return interaction.reply({ embeds: [embed] });

    }

  }
};

async function checks(interaction, djRoleID) {

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

  if (djRoleID) {
    const isDJ = djRoleID && interaction.member.roles.cache.has(djRoleID);
    if (!isDJ) {
      await interaction.reply({ content: `❌ | Bu komutu kullanabilmek için **<@&${djRoleID}> rolüne sahip olmalısın.**`, ephemeral: true });
      return;
    }
  }

  return voiceChannel;
}