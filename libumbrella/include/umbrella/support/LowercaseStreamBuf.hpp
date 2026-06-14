#pragma once

#include <cctype>
#include <iostream>
#include <locale>
#include <streambuf>

namespace umbrella {

template <typename CharT, typename Traits = std::char_traits<CharT>>
class LowercaseStreamBuf : public std::basic_streambuf<CharT, Traits>
{
    using streambuf_type = std::basic_streambuf<CharT, Traits>;
    using int_type       = typename streambuf_type::int_type;
    using char_type      = typename streambuf_type::char_type;
    using traits_type    = typename streambuf_type::traits_type;

   public:
    explicit LowercaseStreamBuf(streambuf_type* sourceBuf)
        : sourceBuf_(sourceBuf)
    { loc_ = std::locale(); }

   protected:
    int_type overflow(int_type ch) override
    {
        if (traits_type::eq_int_type(ch, traits_type::eof())) {
            return traits_type::eof();
        }

        char_type originalCh  = traits_type::to_char_type(ch);
        char_type lowercaseCh = std::tolower(originalCh, loc_);

        if (traits_type::eq_int_type(sourceBuf_->sputc(lowercaseCh),
                                     traits_type::eof())) {
            return traits_type::eof();
        }
        return ch;
    }

    std::streamsize xsputn(const char_type* s,
                           std::streamsize  count) override
    {
        std::streamsize written = 0;
        for (std::streamsize i = 0; i < count; ++i) {
            if (traits_type::eq_int_type(
                    overflow(traits_type::to_int_type(s[i])),
                    traits_type::eof())) {
                break;
            }
            ++written;
        }
        return written;
    }

   private:
    streambuf_type* sourceBuf_;
    std::locale     loc_;
};

}  // namespace umbrella