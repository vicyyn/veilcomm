import TopBarContent from "./TopBarContent";
import BottomBarContent from "./BottomBarContent";
import ChatContent from "./ChatContent";
import Scrollbar from "../components/Scrollbar";
import { Box, styled, useTheme, Stack } from "@mui/material";
import Logs from "./Logs";
import Peers from "./Peers";

const ChatWindow = styled(Box)(
  () => `
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
        flex: 1;
`
);

export default function Messenger() {
  const theme = useTheme();

  return (
    <Stack
      display={"flex"}
      margin={0}
      direction={"row"}
      height={"98vh"}
      sx={{ background: theme.colors.alpha.black[50] }}
    >
      <Box sx={{ flex: 0.3 }}>
        <Logs />
      </Box>
      <ChatWindow>
        <Box
          sx={{
            flex: 1,
            border: `${theme.colors.alpha.black[50]} solid 2px`,
            padding: `${theme.spacing(2)}`,
            alignItems: "center",
          }}
        >
          <TopBarContent />
        </Box>
        <Scrollbar>
          <ChatContent />
        </Scrollbar>
        <BottomBarContent />
      </ChatWindow>
      <Box sx={{ flex: 0.3 }}>
        <Peers />
      </Box>
    </Stack>
  );
}
