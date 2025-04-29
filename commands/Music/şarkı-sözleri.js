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
const { useMainPlayer, useQueue } = require('discord-player');
const lyricsFinder = require('lyrics-finder');

module.exports = {
  category: 'Music',
  cooldown: 30,
  data: new SlashCommandBuilder()
    .setName('şarkı-sözleri')
    .setDescription('Oynatılan şarkının sözlerini görebilirsiniz.')
    .addStringOption(option =>
      option
        .setName('şarkı')
        .setDescription('Şarkı sözlerini adıyla aramak isterseniz.')
        .setRequired(false)
    ),

  run: async (client, interaction) => {

    const queue = useQueue(interaction.guild.id);
    if (!queue || !queue.node.isPlaying()) {
      return interaction.reply({
        embeds: [
          new EmbedBuilder()
            .setColor(client.color)
            .setDescription('❌ | Şu anda hiçbir müzik oynatılmıyor.')
        ], ephemeral: true
      });
    }

    const voiceChannel = await checks(interaction);
    if (!voiceChannel) return;

    const requested = interaction.options.getString('şarkı');
    const track = queue.currentTrack;
    const query = requested?.trim() || `${track.title} ${track.author}`;

    const player = useMainPlayer();
    let searchResults = [];
    try { searchResults = await player.lyrics.search({ q: query }); } catch { }

    const createLyricsEmbed = (title, description, url, thumbnail, authorInfo) => {
      const embed = new EmbedBuilder()
        .setColor(client.color)
        .setTitle(title || 'Şarkı Sözleri')
        .setDescription(description)
        .setFooter({ text: `${interaction.user.username} tarafından istendi.` })
      if (url) embed.setURL(url);
      else embed.setURL(track.url);
      if (thumbnail) embed.setImage(thumbnail);
      else embed.setImage(track.thumbnail)
      if (authorInfo) embed.setAuthor({ name: authorInfo });
      else embed.setAuthor({ name: track.author })
      return embed;
    };

    if (searchResults.length && searchResults[0].plainLyrics) {
      const first = searchResults[0];
      const text = first.plainLyrics.length > 1990 ? first.plainLyrics.slice(0, 1990) + '...' : first.plainLyrics;
      const embed = createLyricsEmbed(
        first.title || query,
        text,
        first.url,
        first.thumbnail,
        first.artist ? { name: first.artist.name, iconURL: first.artist.image, url: first.artist.url } : null
      );
      return interaction.reply({ embeds: [embed], ephemeral: true });
    }

    let found;
    try { found = await lyricsFinder(query, ''); } catch { }
    if (!found) 
      return interaction.reply({ content: ':x: | Bu şarkı için söz bulunamadı. Farklı bir arama deneyin.', ephemeral: true });
    

    const snippet = found.length > 1990 ? found.slice(0, 1990) + '...' : found;
    const embed = createLyricsEmbed(`Şarkı Sözleri: ${query}`, snippet);
    return interaction.reply({ embeds: [embed], ephemeral: true });

  }
};

async function checks(interaction) {
  const vc = interaction.member.voice.channel;
  if (!vc) {
    await interaction.reply({ content: '❌ | Ses kanalında değilsiniz.', ephemeral: true });
    return null;
  }
  const botVc = interaction.guild.members.me.voice.channel;
  if (botVc && botVc.id !== vc.id) {
    await interaction.reply({ content: '❌ | Başka bir ses kanalındayım.', ephemeral: true });
    return null;
  }
  return vc;
}
