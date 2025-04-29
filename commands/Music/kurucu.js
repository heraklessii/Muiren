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

const { SlashCommandBuilder, EmbedBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle, AttachmentBuilder, PermissionFlagsBits } = require("discord.js");
const MusicSetting = require("../../models/MusicSetting");

module.exports = {
    category: "Müzik",
    cooldown: 5,
    data: new SlashCommandBuilder()
        .setName("sistemi-kur")
        .setDescription("Komutu kullandığınız kanala menülü sistemi kurarsınız. (OPSİYONEL)")
        .setDefaultMemberPermissions(PermissionFlagsBits.Administrator),

    run: async (client, interaction) => {

        await interaction.deferReply({ ephemeral: true });

        const attachment = new AttachmentBuilder("././image/muziksistemi.png", { name: "muiren.png" });
        await interaction.channel.send({ files: [attachment] });

        const content = `**__Şarkı Listesi:__**\nOynatılan şarkılar burada yer alacak.`;
        const embed = new EmbedBuilder()
            .setColor(client.color)
            .setImage(process.env.BOT_BANNER_URL)
            .setAuthor({ name: 'Oynatılmıyor...', iconURL: 'https://cdn.discordapp.com/emojis/741605543046807626.gif' })
            .setFooter({ text: `Muiren, HERA tarafından geliştirilmektedir.` })

        const row = new ActionRowBuilder().addComponents(
            new ButtonBuilder().setCustomId('spause').setStyle(ButtonStyle.Secondary).setEmoji('⏯️').setDisabled(true),
            new ButtonBuilder().setCustomId('sprevious').setStyle(ButtonStyle.Secondary).setEmoji('⬅️').setDisabled(true),
            new ButtonBuilder().setCustomId('sstop').setStyle(ButtonStyle.Secondary).setEmoji('⏹').setDisabled(true),
            new ButtonBuilder().setCustomId('sskip').setStyle(ButtonStyle.Secondary).setEmoji('➡️').setDisabled(true),
            new ButtonBuilder().setCustomId('sloop').setStyle(ButtonStyle.Secondary).setEmoji('🔄').setDisabled(true)
        );

        const setupMsg = await interaction.channel.send({ content, embeds: [embed], components: [row] });

        await MusicSetting.findOneAndUpdate({ guildId: interaction.guild.id },
            { guildId: interaction.guild.id, channelId: interaction.channel.id, messageId: setupMsg.id, systemEnabled: true },
            { upsert: true }
        );

        return interaction.followUp({ content: '✅ | Müzik sistemi başarıyla kuruldu.', ephemeral: true });

    }
};
