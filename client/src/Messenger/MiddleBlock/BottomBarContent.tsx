import { Box, Button, Stack, TextField, useTheme } from "@mui/material";
import SendTwoToneIcon from "@mui/icons-material/SendTwoTone";
import { useState } from "react";

function BottomBarContent(props: { sendMessage: (text: string) => any }) {
  const theme = useTheme();
  const [message, setMessage] = useState("");

  return (
    <Stack
      direction={"row"}
      alignItems={"center"}
      gap={1}
      p={2}
      sx={{
        overflow: "hidden",
        background: theme.colors.alpha.white[5],
        borderBottom: `${theme.colors.alpha.black[50]} solid 2px`,
        borderTop: `${theme.colors.alpha.black[50]} solid 2px`,
      }}
    >
      <Box flexGrow={1} display="flex" alignItems="center">
        <TextField
          variant="outlined"
          size="small"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          inputProps={{ style: { color: "white", fontSize: "1rem" } }}
          focused={true}
          placeholder="Write your message here"
          fullWidth
          multiline
        />
      </Box>
      <Box>
        <Button
          startIcon={<SendTwoToneIcon />}
          variant="contained"
          onClick={() => {
            props.sendMessage(message);
            setMessage("");
          }}
        >
          {"Send"}
        </Button>
      </Box>
    </Stack>
  );
}

export default BottomBarContent;
