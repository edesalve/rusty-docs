import React, { useState } from "react";

export const Logs = ({
	buttonFunctions,
	buttonInputKeys,
	lastClickedButton,
	payloads,
	setShowChat,
	setShowLogs,
	showLogs,
}) => {
	const buttonMessages = {
		0: "Parse your repo",
		1: "Document your repo",
		2: "Embed your repo on Qdrant",
		3: "Ask Jon about your repo",
	};
	const guideMessages = {
		0: `This is the starting point of your journey with rusty-docs. Here, you can analyze and parse your Rust 
				repository, gaining insights into its structure and components. Utilize the powerful parsing engine to extract 
				information about structs, functions, traits, and more. Once parsed, you'll have a structured JSON representation 
				of the repository's code elements.`,
		1: `The Document section empowers you to generate documentation for your Rust code effortlessly. Leverage rusty-docs 
				documentation generation feature to create well-organized and detailed documentation for each code element. The 
				generated documentation can be seamlessly inserted into the respective locations in your repository.`,
		2: `In this section, you can create embeddings from your repository code elements and store them on Qdrant.`,
		3: `Open up a dialogue between you and Jon to gain insights, ask about functionalities, and explore the knowledge stored 
				in the repository and the associated knowledge graph.`,
	};
	const howToMessages = [
		[
			`Enter the path to the Rust repository you want to analyze.`,
			`Define the output destination for the parsed data.`,
			`Click the "Parse" button to initiate the parsing process.`,
		],
		[
			`Enter the path to the Rust repository you want to document.`,
			`Define the output destination for the parsed data including documentation.`,
			`Through the "Write in repository" setting, you can choose whether to automatically insert the produced documentation in the 
			 right place in your code.`,
			`Customize your LLM preferences.`,
			`Click the "Generate Documentation" button to create documentation for your Rust code.`,
		],
		[
			`Start your Qdrant server.`,
			`Configure Qdrant (collection name, url) and LLM settings.`,
			`Click the "Create Embeddings" button to generate embeddings for your code elements. For better results do this after completing
			 your repository documention.`,
		],
		[
			`Store the repository embeddings on Qdrant.`,
			`Customize your LLM preferences.`,
			`Click the "Ask the Model" button to show the chat.`,
			`Pose questions about your repository's code and functionalities.`,
		],
	];
	const [logs, setLogs] = useState([]);

	const addToLogs = (message) => {
		setLogs((prevLogs) => [...prevLogs, message]);
	};

	const handleButtonClick = async () => {
		setLogs([]);
		setShowLogs(true);

		const requiredFields = buttonInputKeys[lastClickedButton];
		const missingFields = requiredFields.filter((field) => !payloads[field]);

		if (missingFields.length > 0) {
			setShowChat(false);
			missingFields.forEach((field) => logMissingPayload(field));
			return;
		}

		addToLogs(`Working on it...`);

		if (lastClickedButton !== 3) {
			setShowChat(false);
			// Execute the function and log the result
			try {
				const result = await buttonFunctions[lastClickedButton](payloads);
				logFunctionResult(result);
			} catch (error) {
				addToLogs(`Error: ${error.message}.`);
			}
		} else {
			setShowChat(true);
		}
	};

	const logFunctionResult = (result) => {
		addToLogs(`Execution result: ${result}.`);
	};

	const logMissingPayload = (payloadField) => {
		addToLogs(`Mandatory setting field missing: fill "${payloadField}".`);
	};

	return (
		<div className="bg-docGrey4 rounded p-4">
			<p>{guideMessages[lastClickedButton]}</p>
			<div className="font-bold my-4 text-docYellow text-xl">How to use</div>
			<ul className="list-decimal ml-8">
				{howToMessages[lastClickedButton].map((step, index) => (
					<li key={index}>{step}</li>
				))}
			</ul>
			<div className="flex flex-col items-center">
				<button
					className="border border-docGrey2 hover:border-docGrey1 hover:font-bold mx-5 my-8 p-2 rounded text-docYellow"
					onClick={handleButtonClick}
				>
					{buttonMessages[lastClickedButton]}
				</button>
			</div>
			<div className={`${showLogs ? "" : "hidden"}`}>
				{logs.map((log, index) => (
					<div key={index}>{log}</div>
				))}
			</div>
		</div>
	);
};
