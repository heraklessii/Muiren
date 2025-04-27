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

    // Hemen deferReply ile ephemerallık
    await interaction.deferReply({ ephemeral: true });

    // Arama
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
