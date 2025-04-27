const { SlashCommandBuilder, EmbedBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle, AttachmentBuilder } = require("discord.js");
const MusicSetting = require("../../models/MusicSetting");

module.exports = {
    category: "Müzik",
    cooldown: 5,
    data: new SlashCommandBuilder()
        .setName("kurucu")
        .setDescription("Ashii sunucu sahibi özel komut."),

    run: async (client, interaction) => {
        // Sadece sunucu sahibi veya yönetim yetkisi olanlar
        if (interaction.user.id !== interaction.guild.ownerId && !interaction.member.permissions.has("ManageGuild")) {
            return interaction.reply({ content: ':x: | Bu komutu kullanmaya yetkiniz yok.', ephemeral: true });
        }

        // Zaten kurulmuş mu kontrol edin
        const existing = await MusicSetting.findOne({ guildId: interaction.guild.id });
        if (existing && existing.systemEnabled) {
            return interaction.reply({ content: ':x: | Müzik sistemi zaten kurulu.', ephemeral: true });
        }

        // Setup mesajlarını kanala gönder
        const attachment = new AttachmentBuilder("././image/muziksistemi.png", { name: "muiren.png" });
        await interaction.channel.send({ files: [attachment] });

        const content = `**__Şarkı Listesi:__**\nOynatılan şarkılar burada yer alacak.`;
        const embed = new EmbedBuilder()
            .setColor(client.color)
            .setAuthor({ name: `Henüz bu sunucuda şarkı oynatılmıyor.` })
            .setImage(`https://images2.alphacoders.com/110/thumb-1920-1109233.jpg`)
            .setFooter({ text: `Ashii Müzik Sistemi` });

        const row = new ActionRowBuilder().addComponents(
            new ButtonBuilder().setCustomId('spause').setStyle(ButtonStyle.Secondary).setEmoji('⏯').setDisabled(true),
            new ButtonBuilder().setCustomId('sprevious').setStyle(ButtonStyle.Secondary).setEmoji('⬅').setDisabled(true),
            new ButtonBuilder().setCustomId('sstop').setStyle(ButtonStyle.Secondary).setEmoji('✖').setDisabled(true),
            new ButtonBuilder().setCustomId('sskip').setStyle(ButtonStyle.Secondary).setEmoji('➡').setDisabled(true),
            new ButtonBuilder().setCustomId('sloop').setStyle(ButtonStyle.Secondary).setEmoji('🔄').setDisabled(true)
        );

        const setupMsg = await interaction.channel.send({ content, embeds: [embed], components: [row] });

        // MongoDB'de ayarları kaydet veya güncelle
        await MusicSetting.findOneAndUpdate(
            { guildId: interaction.guild.id },
            {
                guildId: interaction.guild.id,
                channelId: interaction.channel.id,
                messageId: setupMsg.id,
                systemEnabled: true
            },
            { upsert: true }
        );

        return interaction.reply({ content: '✅ | Müzik sistemi başarıyla kuruldu.', ephemeral: true });
    }
};
