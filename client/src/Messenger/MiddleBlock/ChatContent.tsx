import { Box, Card, styled, useTheme } from "@mui/material";
import { ReactNode, useState } from "react";
import { Scrollbars } from "react-custom-scrollbars-2";

const CardWrapperPrimary = styled(Card)(
  ({ theme }) => `
      background: ${theme.colors.primary.main};
      color: ${theme.palette.primary.contrastText};
      padding: ${theme.spacing(2)};
      border-radius: ${theme.general.borderRadiusXl};
      border-top-right-radius: ${theme.general.borderRadius};
      max-width: 380px;
      display: inline-flex;
`
);

const CardWrapperSecondary = styled(Card)(
  ({ theme }) => `
      background: ${theme.palette.grey[200]};
      color: ${theme.colors.alpha.black[100]};
      padding: ${theme.spacing(2)};
      border-radius: ${theme.general.borderRadiusXl};
      border-top-left-radius: ${theme.general.borderRadius};
      max-width: 380px;
      display: inline-flex;
`
);

function ChatContent() {
  const theme = useTheme();
  const [conversation, setConversation] = useState<ReactNode[]>([]);

  const renderLeft = (text: string) => (
    <Box
      key={Math.random()}
      display="flex"
      alignItems="flex-start"
      justifyContent="flex-start"
      py={3}
    >
      <Box
        display="flex"
        alignItems="flex-start"
        flexDirection="column"
        justifyContent="flex-start"
        ml={2}
      >
        <CardWrapperSecondary>{text}</CardWrapperSecondary>
      </Box>
    </Box>
  );

  const renderRight = (text: string) => (
    <Box
      key={Math.random()}
      display="flex"
      alignItems="flex-start"
      justifyContent="flex-end"
      py={3}
    >
      <Box
        display="flex"
        alignItems="flex-end"
        flexDirection="column"
        justifyContent="flex-end"
        mr={2}
      >
        <CardWrapperPrimary>{text}</CardWrapperPrimary>
      </Box>
    </Box>
  );

  return ;
}

export default ChatContent;
