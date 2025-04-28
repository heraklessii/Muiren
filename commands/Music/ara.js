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

const { SlashCommandBuilder, ActionRowBuilder, StringSelectMenuBuilder } = require('discord.js');
const { useMainPlayer } = require('discord-player');
const { QueryType } = require('discord-player');

module.exports = {
  category: 'Music',
  cooldown: 5,
  data: new SlashCommandBuilder()
    .setName('ara')
    .setDescription('Şarkı ara ve sonuçlardan seçim yap.')
    .addStringOption(option =>
      option
        .setName('sarki')
        .setDescription('Aranacak şarkı adı veya bağlantısı')
        .setRequired(true)
    ),

  run: async (client, interaction) => {
    
    const player = useMainPlayer();
    const query = interaction.options.getString('sarki');

    await interaction.deferReply({ ephemeral: true });

    const searchResult = await player.search(query, {
      requestedBy: interaction.user,
      searchEngine: QueryType.AUTO
    });

    if (!searchResult || !searchResult.tracks.length) {
      return interaction.editReply({ content: '❌ | Şarkı bulunamadı!' });
    }

    // İlk 5 sonucu al
    const tracks = searchResult.tracks.slice(0, 5);

    // Select menü oluştur
    const menu = new StringSelectMenuBuilder()
      .setCustomId('ara_select')
      .setPlaceholder('Şarkı seçin...')
      .addOptions(
        tracks.map(track => ({
          label: track.title.substring(0, 100),
          description: track.author ? track.author.substring(0, 100) : undefined,
          value: track.url
        }))
      );

    const row = new ActionRowBuilder().addComponents(menu);

    // Menü ile cevapla
    return interaction.editReply({
      content: '🎶 Aşağıdan çalmak istediğiniz şarkıyı seçin:',
      components: [row]
    });
  }
};
