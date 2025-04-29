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
const { SlashCommandBuilder, PermissionFlagsBits } = require('discord.js');
const MusicSetting = require('../../models/MusicSetting');

module.exports = {
    category: "Müzik",
    cooldown: 5,
    data: new SlashCommandBuilder()
        .setName('dj')
        .setDescription('DJ rolünü ayarla veya kaldır.')
        .setDefaultMemberPermissions(PermissionFlagsBits.Administrator)
        .addSubcommand(sub => sub
            .setName('ayarla')
            .setDescription('DJ rolünü ayarlar.')
            .addRoleOption(opt => opt
                .setName('rol')
                .setDescription('DJ olarak tanımlanacak rol')
                .setRequired(true)))
        .addSubcommand(sub => sub
            .setName('kaldır')
            .setDescription('Mevcut DJ rolünü kaldırır.')),

    run: async (client, interaction) => {

        const sub = interaction.options.getSubcommand();
        const guildId = interaction.guild.id;

        let setting = await MusicSetting.findOne({ guildId });
        if (!setting) setting = await MusicSetting.create({ guildId, djRoleID: null, });

        if (sub === 'ayarla') {

            const role = interaction.options.getRole('rol');
            setting.djRoleID = role.id;
            await setting.save();

            return interaction.reply({
                content: `✅ | DJ rolü başarıyla ${role} olarak ayarlandı.`,
                ephemeral: true
            });

        }

        else if (sub === 'kaldır') {

            if (!setting.djRoleID) {
                return interaction.reply({
                    content: '❌ | DJ rolü zaten ayarlanmamış.',
                    ephemeral: true
                });
            }

            setting.djRoleID = null;
            await setting.save();

            return interaction.reply({
                content: '✅ | DJ rolü kaldırıldı.',
                ephemeral: true
            });
        }

    }
};
