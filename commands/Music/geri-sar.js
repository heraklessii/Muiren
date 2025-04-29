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
    .setName('geri-sar')
    .setDescription('Şarkıyı belirttiğiniz saniye kadar geri sararsınız.')
    .addIntegerOption(option =>
      option
        .setName('süre')
        .setDescription('Geri sarmak istediğiniz saniye cinsinden süre.')
        .setRequired(true)
        .setMinValue(1)
        .setMaxValue(300)
    ),

  run: async (client, interaction) => {

    return;

    let queue = useQueue(interaction.guild.id);
    if (!queue) {
      const noSong = new EmbedBuilder()
        .setColor(client.color)
        .setDescription('❌ | Şu anda hiçbir müzik oynatılmıyor.');
      return interaction.reply({ embeds: [noSong], ephemeral: true });
    }

    const setting = await MusicSetting.findOne({ guildId: interaction.guild.id });
    const djRoleID = setting?.djRoleID;

    // ---- İzin Kontrolleri ---- //
    const voiceChannel = await checks(interaction, queue, djRoleID);
    if (!voiceChannel) return;

    const seconds = interaction.options.getInteger('süre');
    const { current } = queue.node.getTimestamp();
    const currentMs = current * 1000;
    let newPositionMs = currentMs - seconds * 1000;
    newPositionMs = Math.max(newPositionMs, 0);

    try {

      await queue.node.seek(newPositionMs);
      const success = new EmbedBuilder()
        .setColor(client.color)
        .setDescription(`☑️ | Şarkıyı **${seconds}** saniye geri sardım.`);

      return interaction.reply({ embeds: [success] });

    }

    catch (err) {
      const fail = new EmbedBuilder()
        .setColor(client.color)
        .setDescription('❌ | Şarkıyı geri sararken bir hata oluştu.');

      return interaction.reply({ embeds: [fail], ephemeral: true });

    }

  }
};

async function checks(interaction, queue, djRoleID) {

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

  // Kullanıcıda DJ rolü varsa kontrolden muaf olur.
  const isDJ = djRoleID && interaction.member.roles.cache.has(djRoleID);
  const track = queue.currentTrack;
  if (!isDJ && track?.requestedBy?.id !== interaction.user.id) {
    await interaction.reply({ content: '❌ | Bu şarkıyı sen açmadın, bu komutu kullanamazsın.', ephemeral: true });
    return null;
  }

  return voiceChannel;
}