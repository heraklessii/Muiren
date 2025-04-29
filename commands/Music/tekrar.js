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
    .setName("tekrar")
    .setDescription("Şarkıyı ya da sırayı tekrarlatırsınız.")
    .addStringOption(option =>
      option
        .setName('mod')
        .setDescription('Tekrar modunu seçiniz: kapat, şarkı, sıra')
        .setRequired(true)
        .addChoices(
          { name: "Kapat", value: "kapat" },
          { name: "Şarkı", value: "şarkı" },
          { name: "Sıra", value: "sıra" }
        )
    ),

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

    const seçenek = interaction.options.getString("mod");
    let modeNum = 0;
    let modeText = "Kapalı";

    switch (seçenek) {
      case "şarkı":
        modeNum = 1;
        modeText = "Şarkıyı Tekrarla";
        break;
      case "sıra":
        modeNum = 2;
        modeText = "Sırayı Tekrarla";
        break;
    }

    queue.setRepeatMode(modeNum);

    return interaction.reply({
      embeds: [new EmbedBuilder()
        .setColor(client.color)
        .setDescription(`🔁 | Tekrar modu başarıyla \`${modeText}\` olarak ayarlandı.`)],
      ephemeral: false
    });

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