import { Box, Button, Stack, TextField, useTheme } from "@mui/material";
import SendTwoToneIcon from "@mui/icons-material/SendTwoTone";

function BottomBarContent() {
  const theme = useTheme();

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
          inputProps={{ style: { color: "white", fontSize: "1rem" } }}
          focused={true}
          placeholder="Write your message here"
          fullWidth
          multiline
        />
      </Box>
      <Box>
        <Button startIcon={<SendTwoToneIcon />} variant="contained">
          {"Send"}
        </Button>
      </Box>
    </Stack>
  );
}

export default BottomBarContent;
